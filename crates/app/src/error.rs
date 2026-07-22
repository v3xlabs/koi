use std::{num::ParseIntError, sync::Arc};

use thiserror::Error;

use crate::models::network::endpoint::provider::RpcError;

#[derive(Debug, Error)]
pub enum KoiError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("unavailable: {0}")]
    Unavailable(String),
    #[error("internal error: {0}")]
    Internal(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("environment error: {0}")]
    Configuration(#[from] figment::Error),
    #[error("rpc error: {0}")]
    Rpc(#[from] RpcError),
    #[error("parse error: {0}")]
    Parse(#[from] ParseIntError),
    #[error("alloy hex error: {0}")]
    AlloyHex(#[from] alloy::primitives::hex::FromHexError),
    #[error("eth prices error: {0}")]
    EthPrices(#[from] eth_prices::error::EthPricesError),
}

impl KoiError {
    pub fn safe_message(&self) -> String {
        match self {
            Self::InvalidInput(message)
            | Self::NotFound(message)
            | Self::Conflict(message)
            | Self::Unavailable(message) => message.clone(),
            Self::Database(sqlx::Error::RowNotFound) => "resource not found".to_string(),
            _ => "the operation could not be completed".to_string(),
        }
    }
}

impl From<Arc<KoiError>> for KoiError {
    fn from(error: Arc<KoiError>) -> Self {
        match Arc::try_unwrap(error) {
            Ok(error) => error,
            Err(error) => KoiError::Internal(error.to_string()),
        }
    }
}
