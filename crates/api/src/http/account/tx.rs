use futures::{StreamExt, stream};
use poem::Result;
use poem::web::Data;
use poem_openapi::{Object, OpenApi, param::Path, payload::Json};
use serde::{Deserialize, Serialize};

use crate::error::KoiError;
use crate::http::ApiTags;
use crate::http::auth::Auth;
use crate::models::account::Account;
use crate::models::account::identity::AccountIdentity;
use crate::models::account::metadata::WalletType;
use crate::models::network::identity::NetworkIdentity;
use crate::models::tx::{Tx, TxBase};
use crate::state::AppState;
use crate::vendor::safe_wallet::tx::fetch_safewallet_tx;

pub struct AccountTxApi;

#[derive(Serialize, Deserialize, Object)]
pub struct AccountTxResponse {
    pub transactions: Vec<Tx>,
}

#[OpenApi]
impl AccountTxApi {
    /// List all transactions for an account
    ///
    /// GET /api/acc/:account_identity/tx
    #[oai(
        path = "/acc/:account_identity/tx",
        method = "get",
        tag = "ApiTags::Transaction"
    )]
    async fn get_account_tx(
        &self,
        account_identity: Path<AccountIdentity>,
        state: Data<&AppState>,
        auth: Auth,
    ) -> Result<Json<AccountTxResponse>> {
        let _auth_data = auth.unwrap()?;

        let account = Account::get_by_id(&state.database, account_identity.0).await?;

        let transactions: Vec<TxBase> = match account.metadata {
            WalletType::Safe(safe) => {
                stream::iter(account.networks)
                    .map(|network_identity| async move {
                        let txs = fetch_safewallet_tx(network_identity, safe.evm_address.0)
                            .await
                            .unwrap();
                        txs.results
                            .into_iter()
                            .filter_map(|tx| tx.try_into().ok())
                            .collect::<Vec<TxBase>>()
                    })
                    .buffered(8)
                    .collect::<Vec<Vec<TxBase>>>()
                    .await
            }
            _ => vec![],
        }
        .into_iter()
        .flatten()
        .collect::<Vec<TxBase>>();

        let transactions: Vec<Tx> = stream::iter(transactions)
            .map(|tx| {
                let state = state.clone();
                async move { tx.decode(&state).await }
            })
            .buffered(8)
            .collect::<Vec<Result<Tx, KoiError>>>()
            .await
            .into_iter()
            .flat_map(|x| x.ok())
            .collect();

        Ok(Json(AccountTxResponse { transactions }))
    }

    /// List all pending transactions for an account
    ///
    /// GET /api/acc/:account_identity/tx/pending
    #[oai(
        path = "/acc/:account_identity/tx/pending",
        method = "get",
        tag = "ApiTags::Transaction"
    )]
    async fn get_account_tx_pending(
        &self,
        account_identity: Path<AccountIdentity>,
        state: Data<&AppState>,
        auth: Auth,
    ) -> Result<Json<AccountTxResponse>> {
        let _auth_data = auth.unwrap()?;

        let account = Account::get_by_id(&state.database, account_identity.0).await?;

        let address = account
            .metadata
            .unwrap_address()
            .ok_or(KoiError::Internal("Account has no address".to_string()))?;

        // TODO: extrapolate network identity from account
        let network_identity = NetworkIdentity(1);
        // let safewallet_txs = fetch_safewallet_tx_queued(network_identity, address).await?;

        Ok(Json(AccountTxResponse {
            transactions: vec![],
        }))
    }
}
