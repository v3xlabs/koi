use poem::{Result, web::Data};
use poem_openapi::{OpenApi, param::Path, payload::Json};

use super::super::ApiTags;
use crate::{
    http::auth::Auth,
    models::{
        network::{Network, identity::NetworkIdentity},
        tx::{
            decode::{
                DecodeTransactionRequest, DecodeTransactionResponse, SimulateTransactionRequest,
                SimulateTransactionResponse, decode_transaction,
            },
            simulate::simulate_transaction,
        },
    },
    state::AppState,
};

pub struct SimulateApi;

pub fn api() -> impl OpenApi {
    SimulateApi
}

#[OpenApi]
impl SimulateApi {
    /// Simulate a transaction
    ///
    /// POST /api/net/:network_identity/simulate
    #[oai(
        path = "/net/:network_identity/simulate",
        method = "post",
        tag = "ApiTags::Transaction"
    )]
    async fn simulate_transaction(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        network_identity: Path<NetworkIdentity>,
        payload: Json<SimulateTransactionRequest>,
    ) -> Result<Json<SimulateTransactionResponse>> {
        let _auth_data = auth.unwrap()?;

        let _network = Network::get_by_id(&state.database, &network_identity).await?;

        let x = simulate_transaction(&state, &network_identity, &payload.0).await?;

        Ok(Json(x))
    }

    /// Decode a transaction
    ///
    /// POST /api/net/:network_identity/decode
    #[oai(
        path = "/net/:network_identity/decode",
        method = "post",
        tag = "ApiTags::Transaction"
    )]
    async fn decode_transaction(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        network_identity: Path<NetworkIdentity>,
        payload: Json<DecodeTransactionRequest>,
    ) -> Result<Json<DecodeTransactionResponse>> {
        let _auth_data = auth.unwrap()?;

        let _network = Network::get_by_id(&state.database, &network_identity).await?;

        let x = decode_transaction(&state, &network_identity, &payload.0).await?;

        Ok(Json(x))
    }
}
