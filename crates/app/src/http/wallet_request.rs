use crate::{http::auth::Auth, models::wallet_request::FrontendWalletRequest, state::AppState};

use super::ApiTags;
use poem::{Result, web::Data};
use poem_openapi::{Object, OpenApi, param::Path, payload::Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct WalletRequestApi;

pub fn api() -> impl OpenApi {
    WalletRequestApi
}

#[derive(Serialize, Deserialize, Object)]
pub struct WalletRequestsResponse {
    pub requests: Vec<FrontendWalletRequest>,
}

#[derive(Serialize, Deserialize, Object)]
pub struct RejectWalletRequest {
    pub message: Option<String>,
}

#[OpenApi]
impl WalletRequestApi {
    /// List pending wallet requests
    ///
    /// GET /api/wallet-requests
    #[oai(
        path = "/wallet-requests",
        method = "get",
        tag = "ApiTags::WalletRequest"
    )]
    async fn get_wallet_requests(
        &self,
        auth: Auth,
        state: Data<&AppState>,
    ) -> Result<Json<WalletRequestsResponse>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(WalletRequestsResponse {
            requests: state.wallet_requests.all().await,
        }))
    }

    /// Get a pending wallet request
    ///
    /// GET /api/wallet-requests/:request_id
    #[oai(
        path = "/wallet-requests/:request_id",
        method = "get",
        tag = "ApiTags::WalletRequest"
    )]
    async fn get_wallet_request(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        request_id: Path<Uuid>,
    ) -> Result<Json<FrontendWalletRequest>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(state.wallet_requests.get(request_id.0).await?))
    }

    /// Approve a pending wallet request
    ///
    /// POST /api/wallet-requests/:request_id/approve
    #[oai(
        path = "/wallet-requests/:request_id/approve",
        method = "post",
        tag = "ApiTags::WalletRequest"
    )]
    async fn approve_wallet_request(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        request_id: Path<Uuid>,
    ) -> Result<Json<FrontendWalletRequest>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(state.wallet_requests.approve(request_id.0).await?))
    }

    /// Reject a pending wallet request
    ///
    /// POST /api/wallet-requests/:request_id/reject
    #[oai(
        path = "/wallet-requests/:request_id/reject",
        method = "post",
        tag = "ApiTags::WalletRequest"
    )]
    async fn reject_wallet_request(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        request_id: Path<Uuid>,
        payload: Json<RejectWalletRequest>,
    ) -> Result<Json<FrontendWalletRequest>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(
            state
                .wallet_requests
                .reject(request_id.0, payload.0.message)
                .await?,
        ))
    }
}
