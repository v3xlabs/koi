use poem_openapi::NewType;
use serde::{Deserialize, Serialize};
use sqlx::{
    Decode, Encode, Sqlite,
    sqlite::{SqliteTypeInfo, SqliteValueRef},
};

#[derive(Debug, Serialize, Deserialize, Clone, NewType, PartialEq, Hash, Eq, Copy)]
pub struct GroupIdentity(pub u64);

impl sqlx::Type<Sqlite> for GroupIdentity {
    fn type_info() -> SqliteTypeInfo {
        <i64 as sqlx::Type<Sqlite>>::type_info()
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <i64 as sqlx::Type<Sqlite>>::compatible(ty)
    }
}

impl<'r> Decode<'r, Sqlite> for GroupIdentity {
    fn decode(
        value: SqliteValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let id: i64 = <i64 as Decode<Sqlite>>::decode(value)?;
        Ok(GroupIdentity(id as u64))
    }
}

impl<'q> Encode<'q, Sqlite> for GroupIdentity {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <i64 as Encode<Sqlite>>::encode(self.0 as i64, buf)
    }
}

use poem_openapi::Object;
use sqlx::{FromRow, Row, query, query_as, query_scalar, sqlite::SqliteRow};

use crate::{error::KoiError, state::DB};

#[derive(Serialize, Deserialize, Object, Clone)]
pub struct AccountGroup {
    pub group_identity: GroupIdentity,
    pub name: String,
    pub display_order: u32,
}

#[derive(Serialize, Deserialize, Object, Clone)]
pub struct AccountGroupCreate {
    pub name: String,
}

#[derive(Serialize, Deserialize, Object, Clone)]
pub struct AccountGroupUpdate {
    pub name: Option<String>,
}

impl<'r> FromRow<'r, SqliteRow> for AccountGroup {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            group_identity: row.try_get("group_identity")?,
            name: row.try_get("name")?,
            display_order: row.try_get::<i64, _>("display_order")? as u32,
        })
    }
}

impl AccountGroup {
    pub async fn all(database: &DB) -> Result<Vec<AccountGroup>, KoiError> {
        query_as::<_, AccountGroup>(
            "SELECT * FROM account_groups ORDER BY display_order, group_identity",
        )
        .fetch_all(database)
        .await
        .map_err(KoiError::from)
    }

    pub async fn get_by_id(
        database: &DB,
        group_identity: GroupIdentity,
    ) -> Result<AccountGroup, KoiError> {
        query_as::<_, AccountGroup>("SELECT * FROM account_groups WHERE group_identity = ?")
            .bind(group_identity)
            .fetch_one(database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn create(database: &DB, name: String) -> Result<AccountGroup, KoiError> {
        let group_identity = Self::get_next_identity(database).await?;
        let display_order = Self::get_next_display_order(database).await?;

        query_as::<_, AccountGroup>(
            "INSERT INTO account_groups (group_identity, name, display_order) VALUES (?, ?, ?) RETURNING *",
        )
        .bind(group_identity)
        .bind(name)
        .bind(display_order as i64)
        .fetch_one(database)
        .await
        .map_err(KoiError::from)
    }

    pub async fn update(
        database: &DB,
        group_identity: GroupIdentity,
        update: AccountGroupUpdate,
    ) -> Result<AccountGroup, KoiError> {
        let original = Self::get_by_id(database, group_identity).await?;

        query_as::<_, AccountGroup>(
            "UPDATE account_groups SET name = ? WHERE group_identity = ? RETURNING *",
        )
        .bind(update.name.unwrap_or(original.name))
        .bind(group_identity)
        .fetch_one(database)
        .await
        .map_err(KoiError::from)
    }

    pub async fn delete(database: &DB, group_identity: GroupIdentity) -> Result<(), KoiError> {
        query("UPDATE accounts SET group_id = NULL WHERE group_id = ?")
            .bind(group_identity)
            .execute(database)
            .await
            .map_err(KoiError::from)?;

        query("DELETE FROM account_groups WHERE group_identity = ?")
            .bind(group_identity)
            .execute(database)
            .await
            .map_err(KoiError::from)
            .map(|_| ())
    }

    async fn get_next_identity(database: &DB) -> Result<GroupIdentity, KoiError> {
        query_scalar::<_, i64>("SELECT COALESCE(MAX(group_identity), 0) + 1 FROM account_groups")
            .fetch_one(database)
            .await
            .map(|id| GroupIdentity(id as u64))
            .map_err(KoiError::from)
    }

    async fn get_next_display_order(database: &DB) -> Result<u32, KoiError> {
        query_scalar::<_, i64>("SELECT COALESCE(MAX(display_order), -1) + 1 FROM account_groups")
            .fetch_one(database)
            .await
            .map(|order| order as u32)
            .map_err(KoiError::from)
    }
}
