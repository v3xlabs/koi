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
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct AccountBalances {
    pub balances: Vec<AccountBalance>,
    pub total_quote: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub asset: AssetIdentity,
    pub errors: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AccountBalance {
    pub asset_identity: AssetIdentity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_quote: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_quote_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_24h_quote: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_24h_quote_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_quote: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_quote_error: Option<String>,
    pub updated_at: DateTime<Utc>,
}

async fn quote_nom(
    asset: &Asset,
    balance: Result<U256, KoiError>,
    state: &AppState,
    rpc: &DynProvider,
    block: u64,
    asset_out: &AssetIdentity,
) -> AccountBalance {
    let nominal_amount = U256::from(10).pow(U256::from(asset.asset_decimals as u32));
    let (asset_quote, asset_quote_error): (Option<String>, Option<String>) = match state
        .quoters
        .quote_b(rpc, block, &asset.asset_identity, nominal_amount, asset_out)
        .await
    {
        Ok(asset_quote) => (Some(asset_quote.to_string()), None),
        Err(error) => (None, Some(error.to_string())),
    };
    let block_24h = block - (24 * 60 * 60 / 12);
    let (asset_24h_quote, asset_24h_quote_error): (Option<String>, Option<String>) = match state
        .quoters
        .quote_b(
            rpc,
            block_24h,
            &asset.asset_identity,
            nominal_amount,
            asset_out,
        )
        .await
    {
        Ok(asset_24h_quote) => (Some(asset_24h_quote.to_string()), None),
        Err(error) => (None, Some(error.to_string())),
    };
    let (balance, balance_error): (Option<U256>, Option<String>) = match balance {
        Ok(balance) => (Some(balance), None),
        Err(error) => (None, Some(error.to_string())),
    };

    let (balance_quote, balance_quote_error): (Option<String>, Option<String>) = match &balance {
        Some(balance) => {
            match state
                .quoters
                .quote_b(rpc, block, &asset.asset_identity, *balance, asset_out)
                .await
            {
                Ok(balance_quote) => (Some(balance_quote.to_string()), None),
                Err(error) => (None, Some(error.to_string())),
            }
        }
        None => (None, balance_error.clone()),
    };

    AccountBalance {
        asset_identity: asset.asset_identity.clone(),
        balance: balance.map(|b| b.to_string()),
        balance_error,
        asset_quote,
        asset_quote_error,
        asset_24h_quote,
        asset_24h_quote_error,
        balance_quote,
        balance_quote_error,
        updated_at: Utc::now(),
    }
}

impl Account {
    pub async fn fetch_balances(
        &self,
        state: &AppState,
        asset_out: &AssetIdentity,
    ) -> Result<AccountBalances, KoiError> {
        let mut balances = Vec::new();
        let mut errors = Vec::new();

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
            .map_err(|_e| KoiError::Internal("Failed to get block number".to_string()))?;

        let assets = Account::get_joined_assets_by_network_id(
            &state.database,
            &self.account_identity,
            network_identity,
        )
        .await?;

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

        let balances = stream::iter(balances)
            .map(async |(asset, balance)| {
                quote_nom(&asset, balance, state, &rpc, block, &asset_out).await
            })
            .buffer_unordered(8)
            .collect::<Vec<_>>()
            .await;

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
            errors: Vec::new(),
            total_quote: Some(total_quote),
            updated_at: Utc::now(),
            asset: asset_out.clone(),
        })
    }

    pub async fn fetch_asset_balance(
        &self,
        state: &AppState,
        asset_identity: &AssetIdentity,
        asset_out: &AssetIdentity,
    ) -> Result<AccountBalance, KoiError> {
        let asset = Asset::get_by_id(&state.database, asset_identity).await?;
        let network_identity = asset_identity
            .unwrap_network()
            .ok_or(KoiError::Internal("Asset has no network".to_string()))?;

        if !self.networks.contains(&network_identity) {
            return Err(KoiError::Internal(
                "Account is not on the network".to_string(),
            ));
        }

        let provider = state
            .networks
            .get_pool(&network_identity)
            .get_first_rpc(state)
            .await?;
        let rpc = provider
            .get_provider()
            .ok_or(KoiError::Internal("No provider found for RPC".to_string()))?;
        let block = rpc
            .get_block_number()
            .await
            .map_err(|_e| KoiError::Internal("Failed to get block number".to_string()))?;

        let balance = Asset::fetch_balance(state, asset_identity, self).await;

        Ok(quote_nom(&asset, balance, state, &rpc, block, asset_out).await)
    }
}
