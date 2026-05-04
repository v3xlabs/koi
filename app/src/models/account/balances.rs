use crate::{
    error::KoiError,
    models::{
        account::Account,
        asset::{Asset, identity::AssetIdentity},
        network::identity::NetworkIdentity,
    },
    state::AppState,
};
use alloy::{
    primitives::U256,
    providers::{DynProvider, Provider},
};
use chrono::{DateTime, Utc};
use futures::stream::{self, FuturesUnordered, StreamExt};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Object, Clone)]
pub struct AccountBalances {
    pub balances: Vec<AccountBalance>,
    pub total_quote: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub asset: AssetIdentity,
    pub errors: Vec<String>,
}

#[derive(Serialize, Deserialize, Object, Clone)]
pub struct AccountBalance {
    pub asset_identity: AssetIdentity,
    pub balance: Option<String>,
    pub asset_quote: Option<String>,
    pub balance_quote: Option<String>,
    pub updated_at: DateTime<Utc>,
}

async fn quote_nom(
    asset: &Asset,
    balance: Result<U256, KoiError>,
    state: &AppState,
    rpc: &DynProvider,
    block: u64,
    asset_out: &AssetIdentity,
) -> Result<AccountBalance, KoiError> {
    let nominal_amount = U256::from(10).pow(U256::from(asset.asset_decimals as u32));
    let nominal_quote = state
        .quoters
        .quote_b(rpc, block, &asset.asset_identity, asset_out, nominal_amount)
        .await;
    let balance_quote = match &balance {
        Ok(balance) => Ok(state
            .quoters
            .quote_b(rpc, block, &asset.asset_identity, asset_out, *balance)
            .await?),
        Err(error) => Err(error),
    };

    Ok(AccountBalance {
        asset_identity: asset.asset_identity.clone(),
        balance: balance.as_ref().map(|b| b.to_string()).ok(),
        asset_quote: nominal_quote.map(|q| q.to_string()).ok(),
        balance_quote: balance_quote.map(|q| q.to_string()).ok(),
        updated_at: Utc::now(),
    })
}

impl Account {
    pub async fn get_balances(&self, state: &AppState) -> Result<AccountBalances, KoiError> {
        let mut balances = Vec::new();
        let mut errors = Vec::new();
        let asset_out = AssetIdentity::Fiat("usd".to_string());

        let mut tasks = self
            .networks
            .iter()
            .map(async |network_identity| {
                self.get_balance_by_network(state, network_identity, asset_out.clone())
                    .await
            })
            .collect::<FuturesUnordered<_>>();

        let mut total = U256::from(0);
        while let Some(result) = tasks.next().await {
            match result {
                Ok(balance) => {
                    balances.extend(balance.balances);
                    errors.extend(balance.errors);
                    total += balance
                        .total_quote
                        .unwrap_or("0".to_string())
                        .parse::<U256>()
                        .unwrap();
                }
                Err(error) => errors.push(error.to_string()),
            }
        }

        Ok(AccountBalances {
            balances,
            errors,
            total_quote: Some(total.to_string()),
            updated_at: Utc::now(),
            asset: asset_out.clone(),
        })
    }

    async fn get_balance_by_network(
        &self,
        state: &AppState,
        network_identity: &NetworkIdentity,
        asset_out: AssetIdentity,
    ) -> Result<AccountBalances, KoiError> {
        let provider = state
            .networks
            .get_pool(network_identity)
            .get_first_rpc(state)
            .await?;
        let rpc = provider
            .get_provider()
            .ok_or(KoiError::Internal("No provider found for RPC".to_string()))?;
        let block = rpc
            .get_block_number()
            .await
            .map_err(|e| KoiError::Internal("Failed to get block number".to_string()))?;

        let assets = Asset::get_by_network_id(&state.database, network_identity).await?;

        let balances: Vec<(Asset, Result<U256, KoiError>)> = stream::iter(assets)
            .map(async move |asset| {
                (
                    asset.clone(),
                    Asset::fetch_balance(state, &asset.asset_identity, self).await,
                )
            })
            .buffer_unordered(8)
            .collect::<Vec<_>>()
            .await;

        let quotes = stream::iter(balances)
            .map(async |(asset, balance)| {
                quote_nom(&asset, balance, state, &rpc, block, &asset_out).await
            })
            .buffer_unordered(8)
            .collect::<Vec<_>>()
            .await;

        let mut balances = Vec::new();
        let mut errors = Vec::new();
        for quote in quotes {
            match quote {
                Ok(balance) => balances.push(balance),
                Err(error) => errors.push(error.to_string()),
            }
        }

        let total_quote = balances
            .iter()
            .map(|balance| {
                balance
                    .balance_quote
                    .clone()
                    .unwrap_or("0".to_string())
                    .parse::<U256>()
                    .unwrap()
            })
            .sum::<U256>()
            .to_string();

        Ok(AccountBalances {
            balances,
            errors,
            total_quote: Some(total_quote),
            updated_at: Utc::now(),
            asset: asset_out.clone(),
        })
    }
}
