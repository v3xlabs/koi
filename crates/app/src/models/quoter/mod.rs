use std::str::FromStr;

use alloy::primitives::Address;
use chrono::Utc;
use eth_prices::{
    network::NetworkId,
    quoter::{
        AnyQuoter, erc4626::ERC4626Quoter, fixed::FixedQuoter, uniswap_v2::UniswapV2Quoter,
        uniswap_v3::UniswapV3Quoter,
    },
};
use serde::{Deserialize, Serialize};
use sqlx::{
    Decode, Encode, Sqlite, Type,
    prelude::FromRow,
    query_as,
    sqlite::{SqliteTypeInfo, SqliteValueRef},
};
use ts_rs::TS;

use crate::{
    error::KoiError,
    models::{asset::identity::AssetIdentity, network::identity::NetworkIdentity},
    state::DB,
};

pub mod discover;
pub mod man;
pub mod rpc;

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QuoterConfig {
    Fixed(FixedQuoterConfig),
    Erc4626(Erc4626QuoterConfig),
    UniswapV2(UniswapV2QuoterConfig),
    UniswapV3(UniswapV3QuoterConfig),
}

#[derive(Debug, Serialize, Deserialize, TS)]
pub struct FixedQuoterConfig {
    pub price: String,
    pub decimals: u8,
    pub token_in_decimals: u8,
    pub token_out_decimals: u8,
}

#[derive(Debug, Serialize, Deserialize, TS)]
pub struct Erc4626QuoterConfig {}

#[derive(Debug, Serialize, Deserialize, TS)]
pub struct UniswapV2QuoterConfig {
    pub pair_address: String,
}

#[derive(Debug, Serialize, Deserialize, TS)]
pub struct UniswapV3QuoterConfig {
    pub pool_address: String,
}

impl<'r> Decode<'r, Sqlite> for QuoterConfig {
    fn decode(
        value: SqliteValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let s: String = <String as Decode<Sqlite>>::decode(value)?;
        Ok(serde_json::from_str(&s)?)
    }
}

impl Type<Sqlite> for QuoterConfig {
    fn type_info() -> SqliteTypeInfo {
        <String as sqlx::types::Type<Sqlite>>::type_info()
    }
}

impl<'q> Encode<'q, Sqlite> for QuoterConfig {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <&str as Encode<Sqlite>>::encode_by_ref(&serde_json::to_string(self).unwrap().as_str(), buf)
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow, TS)]
pub struct Quoter {
    pub quoter_identity: String,
    pub quoter_name: String,
    pub token_a: AssetIdentity,
    pub token_b: AssetIdentity,
    pub config: QuoterConfig,
    pub enabled: bool,
    pub watch: bool,
}

#[derive(Debug, Serialize, Deserialize, TS)]
pub struct QuoterCreate {
    pub quoter_name: String,
    pub token_a: AssetIdentity,
    pub token_b: AssetIdentity,
    pub config: QuoterConfig,
    pub enabled: bool,
    pub watch: bool,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(optional_fields)]
pub struct QuoterUpdate {
    pub quoter_name: Option<String>,
    pub token_a: Option<AssetIdentity>,
    pub token_b: Option<AssetIdentity>,
    pub config: Option<QuoterConfig>,
    pub enabled: Option<bool>,
    pub watch: Option<bool>,
}

impl Quoter {
    pub async fn all(database: &DB) -> Result<Vec<Quoter>, KoiError> {
        query_as::<_, Quoter>("SELECT * FROM quoters")
            .fetch_all(database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn get_by_network_id(
        database: &DB,
        network_identity: &NetworkIdentity,
    ) -> Result<Vec<Quoter>, KoiError> {
        let pattern = format!("erc20:{}:%", network_identity);
        let pattern2 = format!("native:{}", network_identity);

        query_as::<_, Quoter>("SELECT * FROM quoters WHERE token_a LIKE ? OR token_b LIKE ? OR token_a = ? OR token_b = ?")
            .bind(&pattern)
            .bind(&pattern)
            .bind(&pattern2)
            .bind(&pattern2)
            .fetch_all(database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn get_by_id(database: &DB, quoter_identity: &str) -> Result<Quoter, KoiError> {
        query_as::<_, Quoter>("SELECT * FROM quoters WHERE quoter_identity = ?")
            .bind(quoter_identity)
            .fetch_one(database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn insert(database: &DB, quoter: QuoterCreate) -> Result<Quoter, KoiError> {
        let quoter_identity = Utc::now().timestamp_millis().to_string();
        query_as::<_, Quoter>("INSERT INTO quoters (quoter_identity, quoter_name, token_a, token_b, config, enabled, watch) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING *")
            .bind(quoter_identity)
            .bind(quoter.quoter_name)
            .bind(quoter.token_a)
            .bind(quoter.token_b)
            .bind(quoter.config)
            .bind(quoter.enabled)
            .bind(quoter.watch)
            .fetch_one(database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn update(
        database: &DB,
        quoter_identity: &str,
        quoter: QuoterUpdate,
    ) -> Result<Quoter, KoiError> {
        query_as::<_, Quoter>("UPDATE quoters SET quoter_name = ?, token_a = ?, token_b = ?, config = ?, enabled = ?, watch = ? WHERE quoter_identity = ? RETURNING *")
            .bind(quoter.quoter_name)
            .bind(quoter.token_a)
            .bind(quoter.token_b)
            .bind(quoter.config)
            .bind(quoter.enabled)
            .bind(quoter.watch)
            .bind(quoter_identity)
            .fetch_one(database)
            .await
            .map_err(KoiError::from)
    }
}

impl TryFrom<&Quoter> for AnyQuoter {
    type Error = KoiError;

    fn try_from(val: &Quoter) -> Result<Self, Self::Error> {
        match &val.config {
            QuoterConfig::Fixed(config) => Ok(FixedQuoter {
                fixed_rate: config
                    .price
                    .parse()
                    .map_err(|e| KoiError::Internal(format!("Invalid fixed rate: {}", e)))?,
                token_in: val.token_a.clone().into(),
                token_out: val.token_b.clone().into(),
                token_in_decimals: config.token_in_decimals,
                token_out_decimals: config.token_out_decimals,
                fixed_rate_decimals: config.decimals,
            }
            .into()),
            QuoterConfig::Erc4626(_) => {
                let network_id = val.token_a.unwrap_network().ok_or(KoiError::Internal(
                    "Missing network for ERC4626 quoter".to_string(),
                ))?;
                Ok(ERC4626Quoter {
                    network_id: NetworkId::from(network_id.0),
                    vault_address: val.token_a.clone().into(),
                    token_address: val.token_b.clone().into(),
                }
                .into())
            }
            QuoterConfig::UniswapV2(config) => {
                let (network_id, address) = val.token_a.unwrap_address().ok_or(
                    KoiError::Internal("Missing address for UniswapV2 quoter".to_string()),
                )?;
                let (_, address2) = val.token_b.unwrap_address().ok_or(KoiError::Internal(
                    "Missing address for UniswapV2 quoter".to_string(),
                ))?;
                Ok(UniswapV2Quoter {
                    network_id: NetworkId::from(network_id.0),
                    pair_address: Address::from_str(&config.pair_address)
                        .map_err(|e| KoiError::Internal(format!("Invalid pair address: {}", e)))?,
                    token0: address,
                    token1: address2,
                }
                .into())
            }
            QuoterConfig::UniswapV3(config) => {
                let (network_id, address) = val.token_a.unwrap_address().ok_or(
                    KoiError::Internal("Missing address for UniswapV3 quoter".to_string()),
                )?;
                let (_, address2) = val.token_b.unwrap_address().ok_or(KoiError::Internal(
                    "Missing address for UniswapV3 quoter".to_string(),
                ))?;
                Ok(UniswapV3Quoter {
                    network_id: NetworkId::from(network_id.0),
                    pool_address: Address::from_str(&config.pool_address)
                        .map_err(|e| KoiError::Internal(format!("Invalid pool address: {}", e)))?,
                    token0: address,
                    token1: address2,
                }
                .into())
            }
        }
    }
}
