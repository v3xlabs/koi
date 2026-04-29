use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, query, query_as, query_scalar, sqlite::SqliteRow};

use crate::{
    error::KoiError,
    models::{
        account::{identity::AccountIdentity, metadata::WalletType},
        network::identity::NetworkIdentity,
    },
    state::AppState,
};

pub mod identity;
pub mod metadata;

#[derive(Serialize, Deserialize, Object, Clone)]
pub struct Account {
    pub account_id: AccountIdentity,
    pub name: String,
    pub networks: Vec<NetworkIdentity>,
    pub metadata: WalletType,
}

#[derive(Serialize, Deserialize, Object, Clone)]
pub struct AccountUpdate {
    pub name: Option<String>,
    pub networks: Option<Vec<NetworkIdentity>>,
    pub metadata: Option<WalletType>,
}

impl<'r> FromRow<'r, SqliteRow> for Account {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let account_id: AccountIdentity = row.try_get("account_id")?;
        let name: String = row.try_get("name")?;
        let raw_networks: String = row.try_get("networks")?;
        let networks: Vec<NetworkIdentity> =
            serde_json::from_str(&raw_networks).map_err(|x| sqlx::Error::Decode(Box::new(x)))?;
        let metadata: WalletType = row.try_get("metadata")?;
        Ok(Account {
            account_id,
            name,
            networks,
            metadata,
        })
    }
}

impl Account {
    pub async fn all(state: &AppState) -> Result<Vec<Account>, KoiError> {
        query_as::<_, Account>("SELECT * FROM accounts")
            .fetch_all(&state.database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn get_by_id(
        state: &AppState,
        account_id: AccountIdentity,
    ) -> Result<Account, KoiError> {
        query_as::<_, Account>("SELECT * FROM accounts WHERE account_id = ?")
            .bind(account_id)
            .fetch_one(&state.database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn create(state: &AppState, account: Account) -> Result<Account, KoiError> {
        query_as::<_, Account>(
            "INSERT INTO accounts (account_id, name, networks, metadata) VALUES (?, ?, ?, ?) RETURNING *",
        )
        .bind(account.account_id)
        .bind(account.name)
        .bind(serde_json::to_string(&account.networks).map_err(|x| sqlx::Error::Encode(Box::new(x)))?)
        .bind(account.metadata)
        .fetch_one(&state.database)
        .await
        .map_err(KoiError::from)
    }

    pub async fn get_next_id(state: &AppState) -> Result<AccountIdentity, KoiError> {
        query_scalar::<_, AccountIdentity>("SELECT MAX(account_id) + 1 FROM accounts")
            .fetch_one(&state.database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn delete(state: &AppState, account_id: AccountIdentity) -> Result<(), KoiError> {
        query("DELETE FROM accounts WHERE account_id = ?")
            .bind(account_id)
            .execute(&state.database)
            .await
            .map_err(KoiError::from)
            .map(|_| ())
    }

    pub async fn update(
        state: &AppState,
        account_id: AccountIdentity,
        account: AccountUpdate,
    ) -> Result<Account, KoiError> {
        let original = Self::get_by_id(state, account_id.clone()).await?;

        query_as::<_, Account>("UPDATE accounts SET name = ?, networks = ?, metadata = ? WHERE account_id = ? RETURNING *")
            .bind(account.name.unwrap_or(original.name))
            .bind(serde_json::to_string(&account.networks.unwrap_or(original.networks)).map_err(|x| sqlx::Error::Encode(Box::new(x)))?)
            .bind(serde_json::to_string(&account.metadata.unwrap_or(original.metadata)).map_err(|x| sqlx::Error::Encode(Box::new(x)))?)
            .bind(account_id.0 as i64)
            .fetch_one(&state.database)
            .await
            .map_err(KoiError::from)
    }
}
