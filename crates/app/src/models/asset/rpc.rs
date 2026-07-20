use alloy::primitives::U256;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    Asset, AssetUpdate as AssetUpdateInput, identity::AssetIdentity,
    metadata::AssetMetadataDiscovery,
};
use crate::{
    error::KoiError,
    rpc::{EmptyParams, RpcHandler},
    rpc_method,
    state::AppState,
};

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct AssetParams {
    pub asset_identity: AssetIdentity,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct AssetCreateParams {
    pub input: Asset,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct AssetUpdateParams {
    pub asset_identity: AssetIdentity,
    pub input: AssetUpdateInput,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(optional_fields)]
pub struct AssetQuoteParams {
    pub asset_identity: AssetIdentity,
    #[serde(default)]
    pub display_asset: Option<AssetIdentity>,
}

rpc_method!(AssetList, "asset.list", EmptyParams => Vec<Asset>);
rpc_method!(AssetGet, "asset.get", AssetParams => Asset);
rpc_method!(AssetCreate, "asset.create", AssetCreateParams => Asset);
rpc_method!(AssetUpdate, "asset.update", AssetUpdateParams => Asset);
rpc_method!(AssetDelete, "asset.delete", AssetParams => ());
rpc_method!(AssetDiscoverMetadata, "asset.discoverMetadata", AssetParams => AssetMetadataDiscovery);
rpc_method!(AssetQuote, "asset.quote", AssetQuoteParams => String);

impl RpcHandler for AssetList {
    async fn handle(state: &AppState, _params: EmptyParams) -> Result<Vec<Asset>, KoiError> {
        Asset::all(&state.database).await
    }
}

impl RpcHandler for AssetGet {
    async fn handle(state: &AppState, params: AssetParams) -> Result<Asset, KoiError> {
        Asset::get_by_id(&state.database, &params.asset_identity).await
    }
}

impl RpcHandler for AssetCreate {
    async fn handle(state: &AppState, params: AssetCreateParams) -> Result<Asset, KoiError> {
        Asset::create(&state.database, params.input).await
    }
}

impl RpcHandler for AssetUpdate {
    async fn handle(state: &AppState, params: AssetUpdateParams) -> Result<Asset, KoiError> {
        Asset::update(&state.database, &params.asset_identity, params.input).await
    }
}

impl RpcHandler for AssetDelete {
    async fn handle(state: &AppState, params: AssetParams) -> Result<(), KoiError> {
        Asset::delete(&state.database, &params.asset_identity).await
    }
}

impl RpcHandler for AssetDiscoverMetadata {
    async fn handle(
        state: &AppState,
        params: AssetParams,
    ) -> Result<AssetMetadataDiscovery, KoiError> {
        Asset::fetch_metadata(state, &params.asset_identity).await
    }
}

impl RpcHandler for AssetQuote {
    async fn handle(state: &AppState, params: AssetQuoteParams) -> Result<String, KoiError> {
        let network = params
            .asset_identity
            .unwrap_network()
            .ok_or_else(|| KoiError::InvalidInput("asset has no network".to_string()))?;
        let output = params
            .display_asset
            .unwrap_or_else(|| AssetIdentity::Fiat("usd".to_string()));
        let asset = Asset::get_by_id(&state.database, &params.asset_identity).await?;
        let amount = U256::from(10).pow(U256::from(asset.asset_decimals));
        state
            .quoters
            .quote(state, &network, &params.asset_identity, &output, amount)
            .await
            .map(|quote| quote.to_string())
    }
}
