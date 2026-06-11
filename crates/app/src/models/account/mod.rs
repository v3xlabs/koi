use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, query, query_as, query_scalar, sqlite::SqliteRow};

use crate::{
    error::KoiError,
    models::{
        account::{group::GroupIdentity, identity::AccountIdentity, metadata::WalletType},
        asset::{Asset, identity::AssetIdentity},
        network::identity::NetworkIdentity,
    },
    state::DB,
};

pub mod balance_cache;
pub mod balances;
pub mod group;
pub mod identity;
pub mod layout;
pub mod metadata;

#[derive(Serialize, Deserialize, Object, Clone)]
pub struct Account {
    pub account_identity: AccountIdentity,
    pub name: String,
    pub networks: Vec<NetworkIdentity>,
    pub metadata: WalletType,
    #[serde(default)]
    pub group_id: Option<GroupIdentity>,
    #[serde(default)]
    pub display_order: u32,
}

#[derive(Serialize, Deserialize, Object, Clone)]
pub struct AccountUpdate {
    pub name: Option<String>,
    pub networks: Option<Vec<NetworkIdentity>>,
    pub metadata: Option<WalletType>,
}

impl<'r> FromRow<'r, SqliteRow> for Account {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let account_identity: AccountIdentity = row.try_get("account_identity")?;
        let name: String = row.try_get("name")?;
        let raw_networks: String = row.try_get("networks")?;
        let networks: Vec<NetworkIdentity> =
            serde_json::from_str(&raw_networks).map_err(|x| sqlx::Error::Decode(Box::new(x)))?;
        let metadata: WalletType = row.try_get("metadata")?;
        let group_id: Option<GroupIdentity> = row
            .try_get::<Option<i64>, _>("group_id")
            .ok()
            .flatten()
            .filter(|id| *id > 0)
            .map(|id| GroupIdentity(id as u64));
        let display_order: u32 = row.try_get::<i64, _>("display_order").unwrap_or(0) as u32;
        Ok(Account {
            account_identity,
            name,
            networks,
            metadata,
            group_id,
            display_order,
        })
    }
}

impl Account {
    pub async fn all(database: &DB) -> Result<Vec<Account>, KoiError> {
        Self::all_ordered(database).await
    }

    pub async fn all_ordered(database: &DB) -> Result<Vec<Account>, KoiError> {
        query_as::<_, Account>(
            "SELECT * FROM accounts ORDER BY group_id IS NOT NULL, group_id, display_order, account_identity",
        )
        .fetch_all(database)
        .await
        .map_err(KoiError::from)
    }

    pub async fn get_by_id(
        database: &DB,
        account_identity: AccountIdentity,
    ) -> Result<Account, KoiError> {
        query_as::<_, Account>("SELECT * FROM accounts WHERE account_identity = ?")
            .bind(account_identity)
            .fetch_one(database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn create(database: &DB, account: Account) -> Result<Account, KoiError> {
        let display_order = if account.display_order == 0 {
            Self::get_next_display_order(database, account.group_id).await?
        } else {
            account.display_order
        };

        query_as::<_, Account>(
            "INSERT INTO accounts (account_identity, name, networks, metadata, group_id, display_order) VALUES (?, ?, ?, ?, ?, ?) RETURNING *",
        )
        .bind(account.account_identity)
        .bind(account.name)
        .bind(serde_json::to_string(&account.networks).map_err(|x| sqlx::Error::Encode(Box::new(x)))?)
        .bind(account.metadata)
        .bind(account.group_id)
        .bind(display_order as i64)
        .fetch_one(database)
        .await
        .map_err(KoiError::from)
    }

    async fn get_next_display_order(
        database: &DB,
        group_id: Option<GroupIdentity>,
    ) -> Result<u32, KoiError> {
        let next = match group_id {
            Some(group_id) => {
                query_scalar::<_, i64>(
                    "SELECT COALESCE(MAX(display_order), -1) + 1 FROM accounts WHERE group_id = ?",
                )
                .bind(group_id)
                .fetch_one(database)
                .await
            }
            None => query_scalar::<_, i64>(
                "SELECT COALESCE(MAX(display_order), -1) + 1 FROM accounts WHERE group_id IS NULL",
            )
            .fetch_one(database)
            .await,
        }?;

        Ok(next as u32)
    }

    pub async fn get_next_identity(database: &DB) -> Result<AccountIdentity, KoiError> {
        query_scalar::<_, AccountIdentity>(
            "SELECT COALESCE(MAX(account_identity), 0) + 1 FROM accounts",
        )
        .fetch_one(database)
        .await
        .map_err(KoiError::from)
    }

    pub async fn delete(database: &DB, account_identity: AccountIdentity) -> Result<(), KoiError> {
        query("DELETE FROM accounts WHERE account_identity = ?")
            .bind(account_identity)
            .execute(database)
            .await
            .map_err(KoiError::from)
            .map(|_| ())
    }

    pub async fn update(
        database: &DB,
        account_identity: AccountIdentity,
        account: AccountUpdate,
    ) -> Result<Account, KoiError> {
        let original = Self::get_by_id(database, account_identity.clone()).await?;

        query_as::<_, Account>("UPDATE accounts SET name = ?, networks = ?, metadata = ? WHERE account_identity = ? RETURNING *")
            .bind(account.name.unwrap_or(original.name))
            .bind(serde_json::to_string(&account.networks.unwrap_or(original.networks)).map_err(|x| sqlx::Error::Encode(Box::new(x)))?)
            .bind(serde_json::to_string(&account.metadata.unwrap_or(original.metadata)).map_err(|x| sqlx::Error::Encode(Box::new(x)))?)
            .bind(account_identity.0 as i64)
            .fetch_one(database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn add_asset(
        database: &DB,
        account_identity: AccountIdentity,
        asset_identity: AssetIdentity,
    ) -> Result<(), KoiError> {
        query("INSERT INTO account_assets (account_identity, asset_identity) VALUES (?, ?)")
            .bind(account_identity)
            .bind(asset_identity)
            .execute(database)
            .await
            .map_err(KoiError::from)
            .map(|_| ())
    }

    pub async fn remove_asset(
        database: &DB,
        account_identity: AccountIdentity,
        asset_identity: AssetIdentity,
    ) -> Result<(), KoiError> {
        query("DELETE FROM account_assets WHERE account_identity = ? AND asset_identity = ?")
            .bind(account_identity)
            .bind(asset_identity)
            .execute(database)
            .await
            .map_err(KoiError::from)
            .map(|_| ())
    }

    pub async fn get_assets(
        database: &DB,
        account_identity: AccountIdentity,
    ) -> Result<Vec<AssetIdentity>, KoiError> {
        query_as::<_, AssetIdentity>(
            "SELECT asset_identity FROM account_assets WHERE account_identity = ?",
        )
        .bind(account_identity)
        .fetch_all(database)
        .await
        .map_err(KoiError::from)
    }

    pub async fn get_joined_assets_by_network_id(
        database: &DB,
        account_identity: &AccountIdentity,
        network_identity: &NetworkIdentity,
    ) -> Result<Vec<Asset>, KoiError> {
        query_as::<_, Asset>(
            "SELECT * FROM assets WHERE asset_identity IN (SELECT asset_identity FROM account_assets WHERE account_identity = ? AND (asset_identity LIKE ? OR asset_identity = ?))",
        )
        .bind(account_identity)
        .bind(format!("erc20:{}:%%", network_identity))
        .bind(format!("native:{}", network_identity))
        .fetch_all(database)
        .await
        .map_err(KoiError::from)
    }
}
