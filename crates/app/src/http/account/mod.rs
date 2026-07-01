use crate::{
    http::auth::Auth,
    models::{
        account::{
            Account, AccountUpdate,
            balances::{AccountBalance, AccountBalances},
            group::{AccountGroup, AccountGroupCreate, AccountGroupUpdate, GroupIdentity},
            identity::AccountIdentity,
            layout::{AccountLayout, AccountLayoutUpdate},
        },
        asset::identity::AssetIdentity,
    },
    state::AppState,
};

use super::ApiTags;
use poem::{Result, web::Data};
use poem_openapi::{
    Object, OpenApi,
    param::{Path, Query},
    payload::Json,
};
use serde::{Deserialize, Serialize};

mod derive;
mod tx;

pub struct AccountApi;

pub fn api() -> impl OpenApi {
    (AccountApi, tx::AccountTxApi, derive::AccountDeriveApi)
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

        let accounts = Account::all(&state.database).await?;

        Ok(Json(AccountsResponse { accounts }))
    }

    /// Get an account by ID
    ///
    /// GET /api/acc/:account_identity
    #[oai(
        path = "/acc/:account_identity",
        method = "get",
        tag = "ApiTags::Account"
    )]
    async fn get_account_by_id(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        account_identity: Path<AccountIdentity>,
    ) -> Result<Json<Account>> {
        let _auth_data = auth.unwrap()?;

        let account = Account::get_by_id(&state.database, account_identity.0).await?;

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

        Ok(Json(Account::create(&state.database, payload.0).await?))
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

        Ok(Json(Account::get_next_identity(&state.database).await?))
    }

    /// Delete an account by ID
    ///
    /// DELETE /api/acc/:account_identity
    #[oai(
        path = "/acc/:account_identity",
        method = "delete",
        tag = "ApiTags::Account"
    )]
    async fn delete_account_by_id(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        account_identity: Path<AccountIdentity>,
    ) -> Result<Json<()>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(
            Account::delete(&state.database, account_identity.0).await?,
        ))
    }

    /// Update an account by ID
    ///
    /// PUT /api/acc/:account_identity
    #[oai(
        path = "/acc/:account_identity",
        method = "put",
        tag = "ApiTags::Account"
    )]
    async fn update_account_by_id(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        account_identity: Path<AccountIdentity>,
        payload: Json<AccountUpdate>,
    ) -> Result<Json<Account>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(
            Account::update(&state.database, account_identity.0, payload.0).await?,
        ))
    }

    /// Get the assets of an account
    ///
    /// GET /api/acc/:account_identity/assets
    #[oai(
        path = "/acc/:account_identity/assets",
        method = "get",
        tag = "ApiTags::Account"
    )]
    async fn get_assets_of_account(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        account_identity: Path<AccountIdentity>,
    ) -> Result<Json<Vec<AssetIdentity>>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(
            Account::get_assets(&state.database, account_identity.0).await?,
        ))
    }

    /// Add an asset to an account
    ///
    /// POST /api/acc/:account_identity/asset/:asset_identity
    #[oai(
        path = "/acc/:account_identity/asset/:asset_identity",
        method = "post",
        tag = "ApiTags::Account"
    )]
    async fn add_asset_to_account(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        account_identity: Path<AccountIdentity>,
        asset_identity: Path<AssetIdentity>,
    ) -> Result<Json<()>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(
            Account::add_asset(&state.database, account_identity.0, asset_identity.0).await?,
        ))
    }

    /// Remove an asset from an account
    ///
    /// DELETE /api/acc/:account_identity/asset/:asset_identity
    #[oai(
        path = "/acc/:account_identity/asset/:asset_identity",
        method = "delete",
        tag = "ApiTags::Account"
    )]
    async fn remove_asset_from_account(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        account_identity: Path<AccountIdentity>,
        asset_identity: Path<AssetIdentity>,
    ) -> Result<Json<()>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(
            Account::remove_asset(&state.database, account_identity.0, asset_identity.0).await?,
        ))
    }

    /// Get the balance of a single asset for an account
    ///
    /// GET /api/acc/:account_identity/asset/:asset_identity/balance
    #[oai(
        path = "/acc/:account_identity/asset/:asset_identity/balance",
        method = "get",
        tag = "ApiTags::Account"
    )]
    async fn get_asset_balance_of_account(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        account_identity: Path<AccountIdentity>,
        asset_identity: Path<AssetIdentity>,
        display_currency: Query<AssetIdentity>,
    ) -> Result<Json<AccountBalance>> {
        let _auth_data = auth.unwrap()?;

        let account = Account::get_by_id(&state.database, account_identity.0).await?;

        Ok(Json(
            account
                .fetch_asset_balance(&state, &asset_identity.0, &display_currency.0)
                .await?,
        ))
    }

    /// Get the balances of an account
    ///
    /// GET /api/acc/:account_identity/balances
    #[oai(
        path = "/acc/:account_identity/balances",
        method = "get",
        tag = "ApiTags::Account"
    )]
    async fn get_balances_of_account(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        account_identity: Path<AccountIdentity>,
        display_currency: Query<AssetIdentity>,
        fresh: Query<Option<bool>>,
    ) -> Result<Json<AccountBalances>> {
        let _auth_data = auth.unwrap()?;

        let account = Account::get_by_id(&state.database, account_identity.0).await?;

        Ok(Json(
            state
                .balances
                .get_balances(
                    &state,
                    &account,
                    &display_currency.0,
                    fresh.0.unwrap_or(false),
                )
                .await?,
        ))
    }

    /// Get account layout (groups + ordered accounts)
    ///
    /// GET /api/acc/layout
    #[oai(path = "/acc/layout", method = "get", tag = "ApiTags::Account")]
    async fn get_account_layout(
        &self,
        auth: Auth,
        state: Data<&AppState>,
    ) -> Result<Json<AccountLayout>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(AccountLayout::get(&state.database).await?))
    }

    /// Update account layout (bulk reorder + group membership)
    ///
    /// PUT /api/acc/layout
    #[oai(path = "/acc/layout", method = "put", tag = "ApiTags::Account")]
    async fn update_account_layout(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        payload: Json<AccountLayoutUpdate>,
    ) -> Result<Json<AccountLayout>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(
            AccountLayout::update(&state.database, payload.0).await?,
        ))
    }

    /// Create an account group
    ///
    /// POST /api/acc/groups
    #[oai(path = "/acc/groups", method = "post", tag = "ApiTags::Account")]
    async fn create_account_group(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        payload: Json<AccountGroupCreate>,
    ) -> Result<Json<AccountGroup>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(
            AccountGroup::create(&state.database, payload.0.name).await?,
        ))
    }

    /// Update an account group
    ///
    /// PUT /api/acc/groups/:group_identity
    #[oai(
        path = "/acc/groups/:group_identity",
        method = "put",
        tag = "ApiTags::Account"
    )]
    async fn update_account_group(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        group_identity: Path<GroupIdentity>,
        payload: Json<AccountGroupUpdate>,
    ) -> Result<Json<AccountGroup>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(
            AccountGroup::update(&state.database, group_identity.0, payload.0).await?,
        ))
    }

    /// Delete an account group
    ///
    /// DELETE /api/acc/groups/:group_identity
    #[oai(
        path = "/acc/groups/:group_identity",
        method = "delete",
        tag = "ApiTags::Account"
    )]
    async fn delete_account_group(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        group_identity: Path<GroupIdentity>,
    ) -> Result<Json<()>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(
            AccountGroup::delete(&state.database, group_identity.0).await?,
        ))
    }
}
