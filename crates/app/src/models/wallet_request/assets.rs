use std::{collections::HashMap, str::FromStr, sync::Arc};

use alloy::primitives::Address;
use serde_json::{Value, json};
use tokio::sync::RwLock;

use crate::{
    models::{
        account::identity::AccountIdentity,
        asset::{Asset, identity::AssetIdentity},
        network::identity::NetworkIdentity,
    },
    state::AppState,
};

#[derive(Clone, Default)]
pub struct Assets(Arc<RwLock<HashMap<AccountIdentity, Vec<WatchedAsset>>>>);

#[derive(Clone, Debug)]
pub struct WatchedAsset {
    pub account: AccountIdentity,
    chain: NetworkIdentity,
    address: Address,
    symbol: String,
    decimals: u8,
}

impl Assets {
    pub async fn get(
        &self,
        state: &AppState,
        account: &AccountIdentity,
        account_address: Option<&str>,
        request: &Value,
    ) -> Result<Value, String> {
        let params = request
            .get("params")
            .and_then(|params| {
                params
                    .as_array()
                    .and_then(|items| items.first())
                    .or(Some(params))
            })
            .ok_or_else(|| "wallet_getAssets missing parameters".to_string())?;
        let requested_account = params
            .get("account")
            .and_then(Value::as_str)
            .ok_or_else(|| "wallet_getAssets missing account".to_string())?;

        if account_address.is_some_and(|address| !address.eq_ignore_ascii_case(requested_account)) {
            return Err(
                "wallet_getAssets account does not match the connected account".to_string(),
            );
        }

        let account_address = Address::from_str(requested_account)
            .map_err(|error| format!("invalid account address: {error}"))?;
        let assets = self
            .0
            .read()
            .await
            .get(account)
            .cloned()
            .unwrap_or_default();
        let mut result = serde_json::Map::new();
        for asset in assets {
            if !matches_filters(&asset, params) {
                continue;
            }

            let response = asset.response(state, account_address).await?;

            result
                .entry(format!("0x{:x}", asset.chain.0))
                .or_insert_with(|| json!([]))
                .as_array_mut()
                .unwrap()
                .push(response);
        }

        Ok(Value::Object(result))
    }

    pub async fn watch(&self, asset: &WatchedAsset) {
        let mut assets = self.0.write().await;
        let watched = assets.entry(asset.account.clone()).or_default();
        watched.retain(|current| current.chain != asset.chain || current.address != asset.address);
        watched.push(asset.clone());
    }

    pub fn capabilities_response(network: &NetworkIdentity) -> Value {
        json!({
            format!("0x{:x}", network.0): {
                "assetDiscovery": { "supported": true }
            }
        })
    }
}

impl WatchedAsset {
    async fn response(&self, state: &AppState, account: Address) -> Result<Value, String> {
        let identity = AssetIdentity::ERC20(self.chain.clone(), self.address);
        let balance = Asset::fetch_erc20_balance(state, &identity, account)
            .await
            .map_err(|error| format!("failed to fetch {} balance: {error}", self.symbol))?;

        Ok(json!({
            "address": self.address.to_checksum(None),
            "balance": format!("0x{balance:x}"),
            "type": "erc20",
            "metadata": {
                "name": self.symbol,
                "symbol": self.symbol,
                "decimals": self.decimals,
            },
        }))
    }
}

pub fn watched_asset(
    account: &AccountIdentity,
    network: &NetworkIdentity,
    request: &Value,
) -> Result<WatchedAsset, String> {
    let params = request
        .get("params")
        .and_then(|params| {
            params
                .as_array()
                .and_then(|items| items.first())
                .or(Some(params))
        })
        .ok_or_else(|| "wallet_watchAsset missing parameter".to_string())?;
    let asset_type = params
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| "wallet_watchAsset missing type".to_string())?;
    if !asset_type.eq_ignore_ascii_case("ERC20") {
        return Err(format!("unsupported wallet_watchAsset type: {asset_type}"));
    }

    let options = params
        .get("options")
        .and_then(Value::as_object)
        .ok_or_else(|| "wallet_watchAsset missing options".to_string())?;
    let address = options
        .get("address")
        .and_then(Value::as_str)
        .ok_or_else(|| "wallet_watchAsset missing address".to_string())
        .and_then(|address| {
            Address::from_str(address).map_err(|error| format!("invalid token address: {error}"))
        })?;
    let chain = options
        .get("chainId")
        .map(parse_chain_id)
        .transpose()?
        .unwrap_or_else(|| network.clone());
    let symbol = options
        .get("symbol")
        .and_then(Value::as_str)
        .unwrap_or("TOKEN")
        .to_string();
    let decimals = options
        .get("decimals")
        .and_then(Value::as_u64)
        .and_then(|value| u8::try_from(value).ok())
        .unwrap_or(18);

    Ok(WatchedAsset {
        account: account.clone(),
        chain,
        address,
        symbol,
        decimals,
    })
}

fn matches_filters(asset: &WatchedAsset, params: &Value) -> bool {
    if let Some(filter) = params.get("assetFilter").and_then(Value::as_object) {
        let chain = format!("0x{:x}", asset.chain.0);
        return filter
            .get(&chain)
            .and_then(Value::as_array)
            .is_some_and(|assets| {
                assets.iter().any(|candidate| {
                    candidate
                        .get("type")
                        .and_then(Value::as_str)
                        .is_some_and(|kind| kind.eq_ignore_ascii_case("erc20"))
                        && candidate
                            .get("address")
                            .and_then(Value::as_str)
                            .is_some_and(|address| {
                                address.eq_ignore_ascii_case(&asset.address.to_string())
                            })
                })
            });
    }

    let matches_chain = params
        .get("chainFilter")
        .and_then(Value::as_array)
        .is_none_or(|chains| {
            chains
                .iter()
                .any(|chain| parse_chain_id(chain).is_ok_and(|chain| chain == asset.chain))
        });
    let matches_type = params
        .get("assetTypeFilter")
        .and_then(Value::as_array)
        .is_none_or(|types| {
            types
                .iter()
                .filter_map(Value::as_str)
                .any(|kind| kind.eq_ignore_ascii_case("erc20"))
        });

    matches_chain && matches_type
}

fn parse_chain_id(value: &Value) -> Result<NetworkIdentity, String> {
    if let Some(chain) = value.as_u64() {
        return Ok(NetworkIdentity(chain));
    }

    let chain = value
        .as_str()
        .ok_or_else(|| "chainId must be a number or string".to_string())?;
    let parsed = chain
        .strip_prefix("0x")
        .or_else(|| chain.strip_prefix("0X"))
        .map_or_else(|| chain.parse(), |chain| u64::from_str_radix(chain, 16));
    parsed
        .map(NetworkIdentity)
        .map_err(|error| format!("invalid chainId: {error}"))
}
