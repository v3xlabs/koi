use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use sqlx::query;

use super::{
    group::{AccountGroup, GroupIdentity},
    identity::AccountIdentity,
    Account,
};
use crate::{error::KoiError, state::DB};

#[derive(Serialize, Deserialize, Object, Clone)]
pub struct AccountLayout {
    pub groups: Vec<AccountGroup>,
    pub accounts: Vec<Account>,
}

#[derive(Serialize, Deserialize, Object, Clone)]
pub struct AccountLayoutGroupEntry {
    pub group_identity: GroupIdentity,
    pub name: String,
    pub display_order: u32,
}

#[derive(Serialize, Deserialize, Object, Clone)]
pub struct AccountLayoutAccountEntry {
    pub account_identity: AccountIdentity,
    pub group_id: Option<GroupIdentity>,
    pub display_order: u32,
}

#[derive(Serialize, Deserialize, Object, Clone)]
pub struct AccountLayoutUpdate {
    pub groups: Vec<AccountLayoutGroupEntry>,
    pub accounts: Vec<AccountLayoutAccountEntry>,
}

impl AccountLayout {
    pub async fn get(database: &DB) -> Result<Self, KoiError> {
        Ok(Self {
            groups: AccountGroup::all(database).await?,
            accounts: Account::all_ordered(database).await?,
        })
    }

    pub async fn update(database: &DB, layout: AccountLayoutUpdate) -> Result<Self, KoiError> {
        let mut tx = database.begin().await.map_err(KoiError::from)?;

        for group in layout.groups {
            query("UPDATE account_groups SET name = ?, display_order = ? WHERE group_identity = ?")
                .bind(group.name)
                .bind(group.display_order as i64)
                .bind(group.group_identity)
                .execute(&mut *tx)
                .await
                .map_err(KoiError::from)?;
        }

        for account in layout.accounts {
            query(
                "UPDATE accounts SET group_id = ?, display_order = ? WHERE account_identity = ?",
            )
            .bind(account.group_id)
            .bind(account.display_order as i64)
            .bind(account.account_identity)
            .execute(&mut *tx)
            .await
            .map_err(KoiError::from)?;
        }

        tx.commit().await.map_err(KoiError::from)?;

        Self::get(database).await
    }
}
