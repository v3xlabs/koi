use std::{num::ParseIntError, sync::Arc};

use poem::{IntoResponse, Response, web::headers::ContentType};
use poem_openapi::{Object, payload::Json, types::ToJSON};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::models::network::endpoint::provider::RpcError;

#[derive(Debug, Error)]
pub enum KoiError {
    #[error("internal error: {0}")]
    Internal(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
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

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for KoiError {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .content_type(ContentType::json().to_string())
            .body(
                Json(ErrorResponse {
                    error: self.to_string(),
                })
                .to_json_string(),
            )
    }
}

impl From<KoiError> for poem::Error {
    fn from(error: KoiError) -> Self {
        poem::Error::from_response(error.into_response())
    }
}

impl KoiError {
    pub fn unwrap(self) -> Result<(), poem::Error> {
        Err(self.into())
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
