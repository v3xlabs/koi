use crate::{
    account::Account,
    http::auth::Auth,
    state::AppState,
};

use super::ApiTags;
use poem::{Result, web::Data};
use poem_openapi::{Object, OpenApi, param::Path, payload::Json};
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
    async fn get_accounts(
        &self,
        auth: Auth,
        state: Data<&AppState>,
    ) -> Result<Json<AccountsResponse>> {
        let _auth_data = auth.unwrap()?;

        let accounts = Account::all(&state).await?;

        Ok(Json(AccountsResponse { accounts }))
    }

    /// Get an account by ID
    ///
    /// GET /api/accounts/:account_id
    #[oai(path = "/accounts/:account_id", method = "get", tag = "ApiTags::Account")]
    async fn get_account_by_id(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        account_id: Path<Uuid>,
    ) -> Result<Json<Account>> {
        let _auth_data = auth.unwrap()?;

        let account = Account::get_by_id(&state, account_id.0).await?;

        Ok(Json(account))
    }
}
