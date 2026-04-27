use std::num::ParseIntError;

use poem::{IntoResponse, Response, web::headers::ContentType};
use reqwest::StatusCode;
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
}

impl IntoResponse for KoiError {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .content_type(ContentType::text_utf8().to_string())
            .body(self.to_string())
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
