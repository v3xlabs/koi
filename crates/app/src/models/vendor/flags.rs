use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use sqlx::{
    Decode, Encode, Sqlite, Type,
    sqlite::{SqliteTypeInfo, SqliteValueRef},
};
use strum::{EnumProperty, IntoEnumIterator};
use strum_macros::{EnumIter, EnumProperty};
use tracing::info;

#[derive(Debug, Serialize, Deserialize, EnumIter, EnumProperty, Hash, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum VendorFlag {
    #[strum(props(comment = "Asset Icon Discovery"))]
    AvaraAssetIcons,

    #[strum(props(comment = "Asset Icon Discovery"))]
    ZerionAssetIcons,

    #[strum(props(comment = "Asset Icon Discovery"))]
    SmoldappAssetIcons,

    #[strum(props(comment = "Network Icon Discovery"))]
    SmoldappNetworkIcons,

    #[strum(props(comment = "Network Icon Discovery"))]
    SafewalletNetworkIcons,

    #[strum(props(comment = "Asset Icon Discovery"))]
    SafewalletAssetIcons,

    #[strum(props(comment = "Transaction Service"))]
    SafewalletTransactionsApi,

    #[strum(props(comment = "Asset Icon Discovery", unfinished = "true"))]
    EtherscanAssetIcons,

    #[strum(props(comment = "Link-out to Etherscan for Transaction Hashes"))]
    EtherscanLinkTxHash,

    #[strum(props(comment = "Link-out to Etherscan for Addresses"))]
    EtherscanLinkAddress,

    #[strum(props(comment = "Link-out to Etherscan for Blocks", unfinished = "true"))]
    EtherscanLinkBlock,

    #[strum(props(comment = "Link-out to Etherscan for Contracts", unfinished = "true"))]
    EtherscanLinkContracts,

    #[strum(props(comment = "Asset Icon Discovery"))]
    BlockscoutAssetIcons,

    #[strum(props(comment = "Link-out to Blockscout for Transaction Hashes"))]
    BlockscoutLinkTxHash,

    #[strum(props(comment = "Link-out to Blockscout for Addresses"))]
    BlockscoutLinkAddress,

    #[strum(props(comment = "Link-out to Blockscout for Blocks", unfinished = "true"))]
    BlockscoutLinkBlock,

    #[strum(props(comment = "Link-out to Blockscout for Contracts", unfinished = "true"))]
    BlockscoutLinkContracts,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VendorFlagInfo {
    pub flag: VendorFlag,
    pub comment: String,
    pub unfinished: bool,
}

impl FromStr for VendorFlag {
    type Err = serde_plain::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_plain::from_str(s)
    }
}

impl Display for VendorFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_plain::to_string(self).unwrap())
    }
}

impl VendorFlag {
    pub fn all() -> Vec<VendorFlagInfo> {
        Self::iter()
            .map(|flag| VendorFlagInfo {
                comment: flag.get_str("comment").unwrap_or_default().to_string(),
                unfinished: flag
                    .get_str("unfinished")
                    .unwrap_or_default()
                    .parse::<bool>()
                    .unwrap_or_default(),
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
