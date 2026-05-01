use crate::{
    http::auth::Auth,
    models::{account::{Account, AccountUpdate, identity::AccountIdentity}, asset::identity::AssetIdentity},
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
    /// GET /api/acc/:account_identity
    #[oai(path = "/acc/:account_identity", method = "get", tag = "ApiTags::Account")]
    async fn get_account_by_id(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        account_identity: Path<AccountIdentity>,
    ) -> Result<Json<Account>> {
        let _auth_data = auth.unwrap()?;

        let account = Account::get_by_id(&state, account_identity.0).await?;

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
    async fn get_next_account_identity(
        &self,
        auth: Auth,
        state: Data<&AppState>,
    ) -> Result<Json<AccountIdentity>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(Account::get_next_identity(&state).await?))
    }

    /// Delete an account by ID
    ///
    /// DELETE /api/acc/:account_identity
    #[oai(path = "/acc/:account_identity", method = "delete", tag = "ApiTags::Account")]
    async fn delete_account_by_id(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        account_identity: Path<AccountIdentity>,
    ) -> Result<Json<()>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(Account::delete(&state, account_identity.0).await?))
    }

    /// Update an account by ID
    ///
    /// PUT /api/acc/:account_identity
    #[oai(path = "/acc/:account_identity", method = "put", tag = "ApiTags::Account")]
    async fn update_account_by_id(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        account_identity: Path<AccountIdentity>,
        payload: Json<AccountUpdate>,
    ) -> Result<Json<Account>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(
            Account::update(&state, account_identity.0, payload.0).await?,
        ))
    }

    /// Get the assets of an account
    ///
    /// GET /api/acc/:account_identity/assets
    #[oai(path = "/acc/:account_identity/assets", method = "get", tag = "ApiTags::Account")]
    async fn get_assets_of_account(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        account_identity: Path<AccountIdentity>,
    ) -> Result<Json<Vec<AssetIdentity>>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(Account::get_assets(&state, account_identity.0).await?))
    }

    /// Add an asset to an account
    ///
    /// POST /api/acc/:account_identity/asset/:asset_identity
    #[oai(path = "/acc/:account_identity/asset/:asset_identity", method = "post", tag = "ApiTags::Account")]
    async fn add_asset_to_account(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        account_identity: Path<AccountIdentity>,
        asset_identity: Path<AssetIdentity>,
    ) -> Result<Json<()>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(Account::add_asset(&state, account_identity.0, asset_identity.0).await?))
    }

    /// Remove an asset from an account
    ///
    /// DELETE /api/acc/:account_identity/asset/:asset_identity
    #[oai(path = "/acc/:account_identity/asset/:asset_identity", method = "delete", tag = "ApiTags::Account")]
    async fn remove_asset_from_account(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        account_identity: Path<AccountIdentity>,
        asset_identity: Path<AssetIdentity>,
    ) -> Result<Json<()>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(Account::remove_asset(&state, account_identity.0, asset_identity.0).await?))
    }
}
