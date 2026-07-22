use alloy::primitives::{Address, U256};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{
    error::KoiError,
    models::{
        alloy::{ApiAddress, ApiBytes, ApiU256},
        network::identity::NetworkIdentity,
        tx::{SafeWalletTxExtra, TxBase, TxExtra},
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct SafeWalletTxQueuedResponse {
    pub count: u64,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub results: Vec<SafeWalletTx>,

    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafeWalletTx {
    pub safe: Option<ApiAddress>,
    pub from: Option<ApiAddress>,
    pub to: Option<ApiAddress>,
    pub value: Option<ApiU256>,
    pub data: Option<ApiBytes>,
    pub nonce: Option<u64>,

    #[serde(rename = "transactionHash")]
    pub transaction_hash: Option<ApiBytes>,
    #[serde(rename = "safeTxHash")]
    pub safe_transaction_hash: Option<ApiBytes>,

    pub origin: Option<String>,
    pub executor: Option<ApiAddress>,
    pub proposer: Option<ApiAddress>,

    #[serde(flatten)]
    pub extra: serde_json::Value,
}

/// https://safe-client.safe.global/v1/chains/1/safes/0xAC3EBDC2Dc0Cc20e937C970D46e2A232d3151aef/transactions/queued?
/// https://api.safe.global/tx-service/eth/api/v1/safes/0xAC3EBDC2Dc0Cc20e937C970D46e2A232d3151aef/multisig-transactions
/// https://api.safe.global/tx-service/eth/api/v1/safes/0xAC3EBDC2Dc0Cc20e937C970D46e2A232d3151aef/all-transactions
/// TODO: extrapolate `eth` out of this url and construct a mapping for network identities
///
/// multisig-transactions outputs pending/queued txs
/// all-transactions outputs completed unless specified by query param
pub async fn fetch_safewallet_tx(
    _network_identity: NetworkIdentity,
    address: Address,
) -> Result<SafeWalletTxQueuedResponse, KoiError> {
    let url = format!(
        "https://api.safe.global/tx-service/eth/api/v1/safes/{}/all-transactions",
        address
    );

    debug!("Fetching SafeWallet tx queued from {}", url);

    let response = reqwest::get(&url)
        .await
        .map_err(|e| KoiError::Internal(format!("Failed to fetch SafeWallet tx queued: {}", e)))?;

    let status = response.status();

    debug!("SafeWallet tx status: {}", status);

    if status.is_success() {
        let txs: SafeWalletTxQueuedResponse = response.json().await.map_err(|e| {
            KoiError::Internal(format!("Failed to parse SafeWallet tx queued: {}", e))
        })?;
        Ok(txs)
    } else {
        Err(KoiError::Internal(
            "Failed to fetch SafeWallet tx queued".to_string(),
        ))
    }
}

impl TryInto<TxBase> for SafeWalletTx {
    type Error = KoiError;

    fn try_into(self) -> Result<TxBase, Self::Error> {
        let tx_hash = match self.transaction_hash {
            Some(tx_hash) => tx_hash,
            None => {
                return Err(KoiError::Internal(
                    "Transaction hash is required".to_string(),
                ));
            }
        };

        let from = match self.from {
            Some(from) => Some(from),
            None => self.safe,
        };

        Ok(TxBase {
            network_identity: NetworkIdentity(1),
            tx_hash: tx_hash.into(),
            from,
            to: self.to,
            data: self.data,
            value: self.value.map(|value| value.0).unwrap_or(U256::ZERO).into(),
            extra: TxExtra {
                safe_wallet: Some(SafeWalletTxExtra {
                    nonce: self.nonce,
                    execution_date: None,
                    safe_tx_hash: self.safe_transaction_hash,
                    proposer: self.proposer,
                    executor: self.executor,
                    is_successful: None,
                    is_executed: None,
                    origin: self.origin,
                    extra: self.extra,
                }),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[tokio::test]
    async fn test_fetch_safewallet_tx_queued() {
        let network_identity = NetworkIdentity(1);
        let address = Address::from_str("0xAC3EBDC2Dc0Cc20e937C970D46e2A232d3151aef").unwrap();
        let txs = fetch_safewallet_tx(network_identity, address)
            .await
            .unwrap();
        println!("{:?}", txs);
    }
}
