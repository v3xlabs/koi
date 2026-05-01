use std::{fmt::Display, str::FromStr};

use poem_openapi::{Enum, Object};
use serde::{Deserialize, Serialize};
use sqlx::{
    Decode, Encode, Sqlite, Type,
    sqlite::{SqliteTypeInfo, SqliteValueRef},
};
use strum::{EnumProperty, IntoEnumIterator};
use strum_macros::{EnumIter, EnumProperty};
use tracing::info;

#[derive(
    Debug, Serialize, Deserialize, Enum, EnumIter, EnumProperty, Hash, PartialEq, Eq, Clone,
)]
#[serde(rename_all = "snake_case")]
#[oai(rename_all = "snake_case")]
pub enum VendorFlag {
    #[strum(props(comment = "Asset Icon Discovery"))]
    AvaraTokenLogos,

    #[strum(props(comment = "Asset Icon Discovery"))]
    ZerionTokenLogos,

    #[strum(props(comment = "Asset Icon Discovery"))]
    EtherscanTokenLogos,

    #[strum(props(comment = "Link-out to Etherscan for Transaction Hashes"))]
    EtherscanLinksTxHash,

    #[strum(props(comment = "Link-out to Etherscan for Addresses"))]
    EtherscanLinksAddress,

    #[strum(props(comment = "Link-out to Etherscan for Blocks"))]
    EtherscanLinksBlock,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct VendorFlagInfo {
    pub flag: VendorFlag,
    pub comment: String,
}

impl FromStr for VendorFlag {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        info!("from_str: {}", s);
        serde_json::from_str(s)
    }
}

impl Display for VendorFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl VendorFlag {
    pub fn all() -> Vec<VendorFlagInfo> {
        Self::iter()
            .map(|flag| VendorFlagInfo {
                comment: flag.get_str("comment").unwrap_or_default().to_string(),
                flag,
            })
            .collect()
    }
}

impl<'q> Encode<'q, Sqlite> for VendorFlag {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as sqlx::Database>::ArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <&str as Encode<Sqlite>>::encode_by_ref(&self.to_string().as_str(), buf)
    }
}

impl<'r> Decode<'r, Sqlite> for VendorFlag {
    fn decode(
        value: SqliteValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        info!("decoddd");
        let s: String = <String as Decode<Sqlite>>::decode(value)?;
        info!("decoded: {}", s);
        Ok(VendorFlag::from_str(&s)?)
    }
}

impl Type<Sqlite> for VendorFlag {
    fn type_info() -> SqliteTypeInfo {
        <String as Type<Sqlite>>::type_info()
    }
    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <String as Type<Sqlite>>::compatible(ty)
    }
}
