use crate::models::network::identity::NetworkIdentity;
use crate::{
    http::auth::Auth,
    models::asset::{
        Asset, AssetUpdate, identity::AssetIdentity, metadata::AssetMetadataDiscovery,
    },
    state::AppState,
};

use super::ApiTags;
use alloy::primitives::U256;
use poem::{Result, web::Data};
use poem_openapi::{
    Object, OpenApi,
    param::{Path, Query},
    payload::Json,
};
use serde::{Deserialize, Serialize};

pub struct AssetApi;

pub fn api() -> impl OpenApi {
    AssetApi
}

#[derive(Serialize, Deserialize, Object)]
pub struct AssetsResponse {
    pub assets: Vec<Asset>,
}

#[OpenApi]
impl AssetApi {
    /// List all assets
    ///
    /// GET /api/asset
    #[oai(path = "/asset", method = "get", tag = "ApiTags::Asset")]
    async fn get_assets(&self, auth: Auth, state: Data<&AppState>) -> Result<Json<AssetsResponse>> {
        let _auth_data = auth.unwrap()?;

        let assets = Asset::all(&state.database).await?;

        Ok(Json(AssetsResponse { assets }))
    }

    /// Create an asset
    ///
    /// POST /api/asset
    #[oai(path = "/asset", method = "post", tag = "ApiTags::Asset")]
    async fn create_asset(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        payload: Json<Asset>,
    ) -> Result<Json<Asset>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(Asset::create(&state.database, payload.0).await?))
    }

    /// Get an asset by ID
    ///
    /// GET /api/asset/:asset_identity
    #[oai(
        path = "/asset/:asset_identity",
        method = "get",
        tag = "ApiTags::Asset"
    )]
    async fn get_asset_by_id(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        asset_identity: Path<AssetIdentity>,
    ) -> Result<Json<Asset>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(
            Asset::get_by_id(&state.database, &asset_identity).await?,
        ))
    }

    /// Discover metadata for an asset
    ///
    /// GET /api/asset/:asset_identity/metadata
    #[oai(
        path = "/asset/:asset_identity/metadata",
        method = "get",
        tag = "ApiTags::Asset"
    )]
    async fn discover_metadata_for_asset(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        asset_identity: Path<AssetIdentity>,
    ) -> Result<Json<AssetMetadataDiscovery>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(Asset::fetch_metadata(&state, &asset_identity).await?))
    }

    /// Update an asset by ID
    ///
    /// PUT /api/asset/:asset_identity
    #[oai(
        path = "/asset/:asset_identity",
        method = "put",
        tag = "ApiTags::Asset"
    )]
    async fn update_asset_by_id(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        asset_identity: Path<AssetIdentity>,
        payload: Json<AssetUpdate>,
    ) -> Result<Json<Asset>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(
            Asset::update(&state.database, &asset_identity, payload.0).await?,
        ))
    }

    /// Delete an asset by ID
    ///
    /// DELETE /api/asset/:asset_identity
    #[oai(
        path = "/asset/:asset_identity",
        method = "delete",
        tag = "ApiTags::Asset"
    )]
    async fn delete_asset_by_id(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        asset_identity: Path<AssetIdentity>,
    ) -> Result<Json<()>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(Asset::delete(&state.database, &asset_identity).await?))
    }

    /// Quote an asset
    ///
    /// GET /api/asset/:asset_identity/quote
    #[oai(
        path = "/asset/:asset_identity/quote",
        method = "get",
        tag = "ApiTags::Asset"
    )]
    async fn quote_asset(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        asset_identity: Path<AssetIdentity>,
        display_asset: Query<Option<AssetIdentity>>,
    ) -> Result<Json<String>> {
        let _auth_data = auth.unwrap()?;

        let asset_out = display_asset
            .0
            .unwrap_or(AssetIdentity::Fiat("usd".to_string()));
        let asset = Asset::get_by_id(&state.database, &asset_identity).await?;

        let amount_in = U256::from(1) * U256::from(10).pow(U256::from(asset.asset_decimals as u32));

        let network_identity = asset_identity
            .unwrap_network()
            .unwrap_or(NetworkIdentity(0));
        let quote = state
            .quoters
            .quote(
                &state,
                &network_identity,
                &asset_identity,
                &asset_out,
                amount_in,
            )
            .await?;

        Ok(Json(quote.to_string()))
    }
}
