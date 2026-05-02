use crate::{
    error::KoiError,
    models::{
        network::identity::NetworkIdentity,
        tx::{
            decode::DecodeTransactionRequest, decode::SimulateTransactionRequest,
            decode::SimulateTransactionResponse, decode::decode_transaction,
        },
    },
    state::AppState,
};

pub async fn simulate_transaction(
    state: &AppState,
    network: &NetworkIdentity,
    request: &SimulateTransactionRequest,
) -> Result<SimulateTransactionResponse, KoiError> {
    let request = DecodeTransactionRequest::from(request);
    let decoded = decode_transaction(state, network, &request).await?;

    Ok(SimulateTransactionResponse { call: decoded.call })
}
