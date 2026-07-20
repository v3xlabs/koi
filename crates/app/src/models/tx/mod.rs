use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    error::KoiError,
    models::{
        alloy::{ApiAddress, ApiBytes, ApiU256},
        network::identity::NetworkIdentity,
        tx::decode::{DecodeTransactionRequest, DecodedCall, decode_transaction},
    },
    state::AppState,
};

pub mod decode;
pub mod rpc;
pub mod simulate;

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(optional_fields)]
pub struct Tx {
    pub network_identity: NetworkIdentity,
    pub tx_hash: Option<ApiBytes>,
    pub from: Option<ApiAddress>,
    pub to: Option<ApiAddress>,
    pub data: Option<ApiBytes>,
    pub value: ApiU256,

    pub decoded: Option<DecodedCall>,
    pub extra: TxExtra,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(optional_fields)]
pub struct TxBase {
    pub network_identity: NetworkIdentity,
    pub tx_hash: Option<ApiBytes>,
    pub from: Option<ApiAddress>,
    pub to: Option<ApiAddress>,
    pub data: Option<ApiBytes>,
    pub value: ApiU256,
    pub extra: TxExtra,
}

impl TxBase {
    pub async fn decode(self, state: &AppState) -> Result<Tx, KoiError> {
        let decoded = decode_transaction(
            state,
            &self.network_identity,
            &DecodeTransactionRequest {
                from: self.from,
                to: self.to.unwrap(),
                value: Some(self.value),
                data: self.data.clone(),
            },
        )
        .await
        .ok()
        .map(|response| response.call);
        Ok(Tx {
            network_identity: self.network_identity,
            tx_hash: self.tx_hash,
            from: self.from,
            to: self.to,
            data: self.data,
            value: self.value,
            decoded,
            extra: self.extra,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(optional_fields)]
pub struct TxExtra {
    pub safe_wallet: Option<SafeWalletTxExtra>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(optional_fields)]
pub struct SafeWalletTxExtra {
    pub nonce: Option<u64>,
    pub execution_date: Option<String>,
    pub safe_tx_hash: Option<ApiBytes>,
    pub proposer: Option<ApiAddress>,
    pub executor: Option<ApiAddress>,
    pub is_successful: Option<bool>,
    pub is_executed: Option<bool>,
    pub origin: Option<String>,
    // confirmations: Option<Vec<Confirmation>>,
    #[ts(type = "unknown")]
    pub extra: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, TS)]
pub struct PendingTx {}
