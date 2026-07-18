use std::{fmt::Display, str::FromStr};

use alloy::primitives::Address;
use eth_prices::asset::AssetIdentifier;
use serde::{Deserialize, Serialize};
use sqlx::{
    Decode, Encode, Sqlite,
    sqlite::{SqliteTypeInfo, SqliteValueRef},
};

use crate::{error::KoiError, models::network::identity::NetworkIdentity};

use sqlx::{FromRow, Row, sqlite::SqliteRow};

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
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
            AssetIdentity::Native(network_identity) => write!(f, "native:{}", network_identity),
            AssetIdentity::ERC20(network_identity, address) => {
                write!(f, "erc20:{}:{}", network_identity, address)
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
            ["erc20", network_identity, address] => Ok(AssetIdentity::ERC20(
                NetworkIdentity::from_str(network_identity)?,
                Address::from_str(address)?,
            )),
            ["fiat", code] => Ok(AssetIdentity::Fiat(code.to_string())),
            _ => Err(KoiError::Internal(format!("Invalid asset identity: {}", s))),
        }
    }
}

impl Serialize for AssetIdentity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for AssetIdentity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

impl<'r> FromRow<'r, SqliteRow> for AssetIdentity {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let s: String = row.try_get("asset_identity")?;
        s.parse().map_err(|_| sqlx::Error::RowNotFound)
    }
}

impl AssetIdentity {
    pub fn unwrap_address(&self) -> Option<(NetworkIdentity, Address)> {
        match self {
            AssetIdentity::ERC20(network, address) => Some((network.clone(), *address)),
            _ => None,
        }
    }

    pub fn unwrap_network(&self) -> Option<NetworkIdentity> {
        match self {
            AssetIdentity::ERC20(network, _address) => Some(network.clone()),
            AssetIdentity::Native(network) => Some(network.clone()),
            _ => None,
        }
    }
}

impl From<AssetIdentity> for AssetIdentifier {
    fn from(val: AssetIdentity) -> Self {
        match val {
            AssetIdentity::ERC20(_network, address) => AssetIdentifier::ERC20 { address },
            AssetIdentity::Native(_network) => AssetIdentifier::Native,
            AssetIdentity::Fiat(code) => AssetIdentifier::Fiat { symbol: code },
        }
    }
}
