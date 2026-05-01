use crate::{
    http::auth::Auth,
    models::asset::{Asset, AssetUpdate, identity::AssetIdentity},
    state::AppState,
};

use super::ApiTags;
use poem::{Result, web::Data};
use poem_openapi::{Object, OpenApi, param::Path, payload::Json};
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

        let assets = Asset::all(&state).await?;

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

        Ok(Json(Asset::create(&state, payload.0).await?))
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

        Ok(Json(Asset::get_by_id(&state, &asset_identity).await?))
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
            Asset::update(&state, &asset_identity, payload.0).await?,
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

        Ok(Json(Asset::delete(&state, &asset_identity).await?))
    }
}
