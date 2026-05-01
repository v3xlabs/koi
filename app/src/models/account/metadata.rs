use std::str::FromStr;

use alloy::primitives::Address;
use poem_openapi::{Object, Union};
use serde::{Deserialize, Serialize};
use sqlx::{
    Decode, Encode, FromRow, Row, Sqlite, Type,
    sqlite::{SqliteRow, SqliteTypeInfo, SqliteValueRef},
};

#[derive(Serialize, Deserialize, Union, Clone)]
#[oai(discriminator_name = "type", rename_all = "snake_case")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WalletType {
    Safe(SafeWallet),
    #[serde(rename = "eoa")]
    EOA(EOAWallet),
    View(ViewWallet),
    Railgun(RailgunWallet),
}

#[derive(Serialize, Deserialize, Object, Clone)]
pub struct SafeWallet {
    pub evm_address: String,
}

#[derive(Serialize, Deserialize, Object, Clone)]
pub struct EOAWallet {
    pub evm_address: String,
}

#[derive(Serialize, Deserialize, Object, Clone)]
pub struct ViewWallet {
    pub evm_address: String,
}

#[derive(Serialize, Deserialize, Object, Clone)]
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

impl ToString for WalletType {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl WalletType {
    pub fn unwrap_address(&self) -> Option<Address> {
        let address = match self {
            WalletType::EOA(EOAWallet { evm_address }) => Some(evm_address),
            WalletType::Safe(SafeWallet { evm_address }) => Some(evm_address),
            WalletType::View(ViewWallet { evm_address }) => Some(evm_address),
            WalletType::Railgun(RailgunWallet { railgun_address }) => Some(railgun_address),
        };

        address.map(|address| Address::from_str(address).unwrap())
    }
}