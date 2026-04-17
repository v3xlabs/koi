use crate::{
    http::auth::Auth,
    account::{Account, EOAWallet, WalletType},
};

use super::ApiTags;
use poem::Result;
use poem_openapi::{Object, OpenApi, payload::Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct AccountsApi;

pub fn api() -> impl OpenApi {
    AccountsApi
}

#[derive(Serialize, Deserialize, Object)]
pub struct AccountsResponse {
    pub accounts: Vec<Account>,
}

#[OpenApi]
impl AccountsApi {
    /// List all accounts
    ///
    /// GET /api/accounts
    #[oai(path = "/accounts", method = "get", tag = "ApiTags::Account")]
    async fn get_accounts(&self, auth: Auth) -> Result<Json<AccountsResponse>> {
        let auth_data = auth.unwrap()?;

        let accounts = vec![Account {
            account_id: Uuid::new_v4(),
            name: "Wallet 1".to_string(),
            chains: vec!["Ethereum".to_string()],
            wallet: WalletType::EOA(EOAWallet {
                evm_address: "0x1234567890123456789012345678901234567890".to_string(),
            }),
        }];

        Ok(Json(AccountsResponse { accounts }))
    }
}
