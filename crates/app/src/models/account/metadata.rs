use std::{fmt::Display, str::FromStr};

use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
use sqlx::{
    Decode, Encode, FromRow, Row, Sqlite, Type,
    sqlite::{SqliteRow, SqliteTypeInfo, SqliteValueRef},
};

use crate::models::alloy::ApiAddress;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WalletType {
    Safe(SafeWallet),
    #[serde(rename = "eoa")]
    EOA(EOAWallet),
    View(ViewWallet),
    Railgun(RailgunWallet),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SafeWallet {
    pub evm_address: ApiAddress,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EOAWallet {
    pub evm_address: ApiAddress,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ViewWallet {
    pub evm_address: ApiAddress,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RailgunWallet {
    pub railgun_address: String,
}

impl<'r> FromRow<'r, SqliteRow> for WalletType {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let s: String = row.try_get("metadata")?;
        let value: WalletType =
            serde_json::from_str(&s).map_err(|x| sqlx::Error::Decode(Box::new(x)))?;
        Ok(value)
    }
}

impl<'r> Decode<'r, Sqlite> for WalletType {
    fn decode(
        value: SqliteValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let s: String = <String as Decode<Sqlite>>::decode(value)?;
        Ok(s.parse()?)
    }
}

impl<'q> Encode<'q, Sqlite> for WalletType {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = self.to_string();
        <&str as Encode<Sqlite>>::encode_by_ref(&s.as_str(), buf)
    }
}

impl Type<Sqlite> for WalletType {
    fn type_info() -> SqliteTypeInfo {
        <String as Type<Sqlite>>::type_info()
    }
    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <String as Type<Sqlite>>::compatible(ty)
    }
}

impl FromStr for WalletType {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: WalletType = serde_json::from_str(s)?;
        Ok(value)
    }
}

impl Display for WalletType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl WalletType {
    pub fn unwrap_address(&self) -> Option<Address> {
        match self {
            WalletType::EOA(EOAWallet { evm_address }) => Some(evm_address.0),
            WalletType::Safe(SafeWallet { evm_address }) => Some(evm_address.0),
            WalletType::View(ViewWallet { evm_address }) => Some(evm_address.0),
            WalletType::Railgun(_) => None,
        }
    }
}
