use identity::AssetIdentity;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query, query_as};
use ts_rs::TS;

use crate::{error::KoiError, models::network::identity::NetworkIdentity, state::DB};

pub mod balances;
pub mod erc20;
pub mod identity;
pub mod metadata;
pub mod rpc;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, TS)]
#[ts(optional_fields)]
pub struct Asset {
    pub asset_identity: AssetIdentity,
    pub asset_name: String,
    pub asset_symbol: String,
    pub asset_decimals: u8,
    pub asset_icon_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(optional_fields)]
pub struct AssetUpdate {
    pub asset_name: Option<String>,
    pub asset_symbol: Option<String>,
    pub asset_decimals: Option<u8>,
    pub asset_icon_url: Option<String>,
}

impl Asset {
    pub async fn all(database: &DB) -> Result<Vec<Asset>, KoiError> {
        query_as::<_, Asset>("SELECT * FROM assets")
            .fetch_all(database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn get_by_id(
        database: &DB,
        asset_identity: &AssetIdentity,
    ) -> Result<Asset, KoiError> {
        query_as::<_, Asset>("SELECT * FROM assets WHERE asset_identity = ?")
            .bind(asset_identity)
            .fetch_one(database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn get_by_network_id(
        database: &DB,
        network_identity: &NetworkIdentity,
    ) -> Result<Vec<Asset>, KoiError> {
        let pattern = format!("erc20:{}:%", network_identity);
        let pattern2 = format!("native:{}", network_identity);

        query_as::<_, Asset>(
            "SELECT * FROM assets WHERE asset_identity LIKE ? OR asset_identity = ?",
        )
        .bind(&pattern)
        .bind(&pattern2)
        .fetch_all(database)
        .await
        .map_err(KoiError::from)
    }

    pub async fn create(database: &DB, asset: Asset) -> Result<Asset, KoiError> {
        query_as::<_, Asset>("INSERT INTO assets (asset_identity, asset_name, asset_symbol, asset_decimals, asset_icon_url) VALUES (?, ?, ?, ?, ?) RETURNING *")
            .bind(asset.asset_identity)
            .bind(asset.asset_name)
            .bind(asset.asset_symbol)
            .bind(asset.asset_decimals)
            .bind(asset.asset_icon_url)
            .fetch_one(database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn update(
        database: &DB,
        asset_identity: &AssetIdentity,
        asset: AssetUpdate,
    ) -> Result<Asset, KoiError> {
        query_as::<_, Asset>("UPDATE assets SET asset_name = ?, asset_symbol = ?, asset_decimals = ?, asset_icon_url = ? WHERE asset_identity = ? RETURNING *")
            .bind(asset.asset_name)
            .bind(asset.asset_symbol)
            .bind(asset.asset_decimals)
            .bind(asset.asset_icon_url)
            .bind(asset_identity)
            .fetch_one(database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn delete(database: &DB, asset_identity: &AssetIdentity) -> Result<(), KoiError> {
        query("DELETE FROM assets WHERE asset_identity = ?")
            .bind(asset_identity)
            .execute(database)
            .await
            .map_err(KoiError::from)
            .map(|_| ())
    }
}
