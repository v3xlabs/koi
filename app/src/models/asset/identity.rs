use std::{fmt::Display, str::FromStr};

use alloy::primitives::Address;
use poem_openapi::types::Example;
use serde::{Deserialize, Serialize};
use sqlx::{
    Decode, Encode, Sqlite,
    sqlite::{SqliteTypeInfo, SqliteValueRef},
};

use crate::{error::KoiError, models::network::identity::NetworkIdentity};

use poem_openapi::{
    registry::MetaSchemaRef,
    types::{ParseError, ParseFromJSON, ParseFromParameter, ParseResult, ToJSON},
};
use serde_json::Value;
use sqlx::{FromRow, Row, sqlite::SqliteRow};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Hash, Eq)]
pub enum AssetIdentity {
    Native(NetworkIdentity),
    ERC20(NetworkIdentity, Address),
    // Fiat Currency
    // ISO 4217 Code
    Fiat(String),
}

impl sqlx::types::Type<Sqlite> for AssetIdentity {
    fn type_info() -> SqliteTypeInfo {
        <String as sqlx::types::Type<Sqlite>>::type_info()
    }
    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <String as sqlx::types::Type<Sqlite>>::compatible(ty)
    }
}
impl<'r> Decode<'r, Sqlite> for AssetIdentity {
    fn decode(
        value: SqliteValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let s: String = <String as Decode<Sqlite>>::decode(value)?;
        Ok(AssetIdentity::from_str(&s)?)
    }
}

impl<'q> Encode<'q, Sqlite> for AssetIdentity {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = self.to_string();
        <&str as Encode<Sqlite>>::encode_by_ref(&s.as_str(), buf)
    }
}

impl Display for AssetIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetIdentity::Native(network_id) => write!(f, "native:{}", network_id),
            AssetIdentity::ERC20(network_id, address) => {
                write!(f, "erc20:{}:{}", network_id, address)
            }
            AssetIdentity::Fiat(code) => write!(f, "fiat:{}", code),
        }
    }
}

impl FromStr for AssetIdentity {
    type Err = KoiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split(":").collect::<Vec<&str>>();
        match parts.as_slice() {
            ["native", network_id] => Ok(AssetIdentity::Native(NetworkIdentity::from_str(
                network_id,
            )?)),
            ["erc20", network_id, address] => Ok(AssetIdentity::ERC20(
                NetworkIdentity::from_str(network_id)?,
                Address::from_str(address)?,
            )),
            ["fiat", code] => Ok(AssetIdentity::Fiat(code.to_string())),
            _ => Err(KoiError::Internal(format!("Invalid asset identity: {}", s))),
        }
    }
}

impl Example for AssetIdentity {
    fn example() -> Self {
        Self::ERC20(
            NetworkIdentity(1),
            Address::from_str("0x0000000000000000000000000000000000000000").unwrap(),
        )
    }
}

impl poem_openapi::types::Type for AssetIdentity {
    const IS_REQUIRED: bool = true;

    type RawValueType = String;

    type RawElementValueType = String;

    fn name() -> std::borrow::Cow<'static, str> {
        "NetworkIdentity".into()
    }

    fn schema_ref() -> MetaSchemaRef {
        <String as poem_openapi::types::Type>::schema_ref()
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        None
    }

    fn raw_element_iter<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &'a Self::RawElementValueType> + 'a> {
        todo!()
    }
}

impl ParseFromJSON for AssetIdentity {
    fn parse_from_json(value: Option<Value>) -> ParseResult<Self> {
        match value {
            Some(Value::String(s)) => Ok(s.parse()?),
            _ => Err(ParseError::custom("Invalid asset identity")),
        }
    }
}

impl ToJSON for AssetIdentity {
    fn to_json(&self) -> Option<Value> {
        Some(Value::String(self.to_string()))
    }
}

impl<'r> FromRow<'r, SqliteRow> for AssetIdentity {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let s: String = row.try_get("asset_identity")?;
        s.parse().map_err(|_| sqlx::Error::RowNotFound)
    }
}

impl ParseFromParameter for AssetIdentity {
    fn parse_from_parameter(value: &str) -> ParseResult<Self> {
        Ok(value.parse()?)
    }
}
