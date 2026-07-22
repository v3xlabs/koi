use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::decode::{
    DecodeTransactionRequest, DecodeTransactionResponse, SimulateTransactionRequest,
    SimulateTransactionResponse, decode_transaction,
};
use super::simulate::simulate_transaction;
use crate::{
    error::KoiError,
    models::network::{Network, identity::NetworkIdentity},
    rpc::RpcHandler,
    rpc_method,
    state::AppState,
};

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct SimulateParams {
    pub network_identity: NetworkIdentity,
    pub input: SimulateTransactionRequest,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct DecodeParams {
    pub network_identity: NetworkIdentity,
    pub input: DecodeTransactionRequest,
}

rpc_method!(TransactionSimulate, "transaction.simulate", SimulateParams => SimulateTransactionResponse);
rpc_method!(TransactionDecode, "transaction.decode", DecodeParams => DecodeTransactionResponse);

impl RpcHandler for TransactionSimulate {
    async fn handle(
        state: &AppState,
        params: SimulateParams,
    ) -> Result<SimulateTransactionResponse, KoiError> {
        Network::get_by_id(&state.database, &params.network_identity).await?;
        simulate_transaction(state, &params.network_identity, &params.input).await
    }
}

impl RpcHandler for TransactionDecode {
    async fn handle(
        state: &AppState,
        params: DecodeParams,
    ) -> Result<DecodeTransactionResponse, KoiError> {
        Network::get_by_id(&state.database, &params.network_identity).await?;
        decode_transaction(state, &params.network_identity, &params.input).await
    }
}
