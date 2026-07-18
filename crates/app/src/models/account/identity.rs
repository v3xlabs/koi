use std::{fmt::Display, num::ParseIntError, str::FromStr};

use serde::{Deserialize, Serialize};
use sqlx::{
    Decode, Encode, Sqlite,
    sqlite::{SqliteTypeInfo, SqliteValueRef},
};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Hash, Eq, TS)]
#[ts(type = "number")]
pub struct AccountIdentity(pub u64);

impl sqlx::types::Type<Sqlite> for AccountIdentity {
    fn type_info() -> SqliteTypeInfo {
        <u64 as sqlx::types::Type<Sqlite>>::type_info()
    }
    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <u64 as sqlx::types::Type<Sqlite>>::compatible(ty)
    }
}

impl<'r> Decode<'r, Sqlite> for AccountIdentity {
    fn decode(
        value: SqliteValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let s: u64 = <u64 as Decode<Sqlite>>::decode(value)?;
        Ok(AccountIdentity(s))
    }
}

impl<'q> Encode<'q, Sqlite> for AccountIdentity {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = self.0.to_string();
        <&str as Encode<Sqlite>>::encode_by_ref(&s.as_str(), buf)
    }
}

impl Display for AccountIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for AccountIdentity {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(AccountIdentity(s.parse::<u64>()?))
    }
}
