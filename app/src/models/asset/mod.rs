use identity::AssetIdentity;
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query, query_as};

use crate::{error::KoiError, state::AppState};

pub mod identity;
pub mod metadata;

#[derive(Debug, Serialize, Deserialize, Object, FromRow)]
pub struct Asset {
    pub asset_identity: AssetIdentity,
    pub asset_name: String,
    pub asset_symbol: String,
    pub asset_decimals: u8,
    pub asset_icon_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct AssetUpdate {
    pub asset_name: Option<String>,
    pub asset_symbol: Option<String>,
    pub asset_decimals: Option<u8>,
    pub asset_icon_url: Option<String>,
}

impl Asset {
    pub async fn all(state: &AppState) -> Result<Vec<Asset>, KoiError> {
        query_as::<_, Asset>("SELECT * FROM assets")
            .fetch_all(&state.database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn get_by_id(state: &AppState, asset_identity: &AssetIdentity) -> Result<Asset, KoiError> {
        query_as::<_, Asset>("SELECT * FROM assets WHERE asset_identity = ?")
            .bind(asset_identity)
            .fetch_one(&state.database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn create(state: &AppState, asset: Asset) -> Result<Asset, KoiError> {
        query_as::<_, Asset>("INSERT INTO assets (asset_identity, asset_name, asset_symbol, asset_decimals, asset_icon_url) VALUES (?, ?, ?, ?, ?) RETURNING *")
            .bind(asset.asset_identity)
            .bind(asset.asset_name)
            .bind(asset.asset_symbol)
            .bind(asset.asset_decimals)
            .bind(asset.asset_icon_url)
            .fetch_one(&state.database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn update(
        state: &AppState,
        asset_identity: &AssetIdentity,
        asset: AssetUpdate,
    ) -> Result<Asset, KoiError> {
        query_as::<_, Asset>("UPDATE assets SET asset_name = ?, asset_symbol = ?, asset_decimals = ?, asset_icon_url = ? WHERE asset_identity = ? RETURNING *")
            .bind(asset.asset_name)
            .bind(asset.asset_symbol)
            .bind(asset.asset_decimals)
            .bind(asset.asset_icon_url)
            .bind(asset_identity)
            .fetch_one(&state.database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn delete(state: &AppState, asset_identity: &AssetIdentity) -> Result<(), KoiError> {
        query("DELETE FROM assets WHERE asset_identity = ?")
            .bind(asset_identity)
            .execute(&state.database)
            .await
            .map_err(KoiError::from)
            .map(|_| ())
    }
}
