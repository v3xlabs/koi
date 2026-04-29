use crate::{
    http::auth::Auth,
    models::account::{Account, AccountUpdate, identity::AccountIdentity},
    state::AppState,
};

use super::ApiTags;
use poem::{Result, web::Data};
use poem_openapi::{Object, OpenApi, param::Path, payload::Json};
use serde::{Deserialize, Serialize};

pub struct AccountApi;

pub fn api() -> impl OpenApi {
    AccountApi
}

#[derive(Serialize, Deserialize, Object)]
pub struct AccountsResponse {
    pub accounts: Vec<Account>,
}

#[OpenApi]
impl AccountApi {
    /// List all accounts
    ///
    /// GET /api/acc
    #[oai(path = "/acc", method = "get", tag = "ApiTags::Account")]
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
    /// GET /api/acc/:account_id
    #[oai(path = "/acc/:account_id", method = "get", tag = "ApiTags::Account")]
    async fn get_account_by_id(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        account_id: Path<AccountIdentity>,
    ) -> Result<Json<Account>> {
        let _auth_data = auth.unwrap()?;

        let account = Account::get_by_id(&state, account_id.0).await?;

        Ok(Json(account))
    }

    /// Create an account
    ///
    /// POST /api/acc
    #[oai(path = "/acc", method = "post", tag = "ApiTags::Account")]
    async fn create_account(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        payload: Json<Account>,
    ) -> Result<Json<Account>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(Account::create(&state, payload.0).await?))
    }

    /// Get the next account ID
    ///
    /// GET /api/acc/next-id
    #[oai(path = "/acc/next-id", method = "get", tag = "ApiTags::Account")]
    async fn get_next_account_id(
        &self,
        auth: Auth,
        state: Data<&AppState>,
    ) -> Result<Json<AccountIdentity>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(Account::get_next_id(&state).await?))
    }

    /// Delete an account by ID
    ///
    /// DELETE /api/acc/:account_id
    #[oai(path = "/acc/:account_id", method = "delete", tag = "ApiTags::Account")]
    async fn delete_account_by_id(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        account_id: Path<AccountIdentity>,
    ) -> Result<Json<()>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(Account::delete(&state, account_id.0).await?))
    }

    /// Update an account by ID
    ///
    /// PUT /api/acc/:account_id
    #[oai(path = "/acc/:account_id", method = "put", tag = "ApiTags::Account")]
    async fn update_account_by_id(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        account_id: Path<AccountIdentity>,
        payload: Json<AccountUpdate>,
    ) -> Result<Json<Account>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(
            Account::update(&state, account_id.0, payload.0).await?,
        ))
    }
}
