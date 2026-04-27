use std::{fmt::Display, num::ParseIntError, str::FromStr};

use poem_openapi::NewType;
use serde::{Deserialize, Serialize};
use sqlx::{
    Decode, Encode, Sqlite,
    sqlite::{SqliteTypeInfo, SqliteValueRef},
};

#[derive(Debug, Serialize, Deserialize, Clone, NewType, PartialEq, Hash, Eq)]
pub struct NetworkIdentity(pub u64);

impl sqlx::types::Type<Sqlite> for NetworkIdentity {
    fn type_info() -> SqliteTypeInfo {
        <u64 as sqlx::types::Type<Sqlite>>::type_info()
    }
    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <u64 as sqlx::types::Type<Sqlite>>::compatible(ty)
    }
}
impl<'r> Decode<'r, Sqlite> for NetworkIdentity {
    fn decode(
        value: SqliteValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let s: u64 = <u64 as Decode<Sqlite>>::decode(value)?;
        Ok(NetworkIdentity(s))
    }
}

impl<'q> Encode<'q, Sqlite> for NetworkIdentity {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = self.0.to_string();
        <&str as Encode<Sqlite>>::encode_by_ref(&s.as_str(), buf)
    }
}

impl Display for NetworkIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for NetworkIdentity {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(NetworkIdentity(s.parse::<u64>()?))
    }
}
