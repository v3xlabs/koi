use poem_openapi::Enum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Enum)]
#[serde(rename_all = "snake_case")]
#[oai(rename_all = "snake_case")]
pub enum VendorFlag {
    AvaraTokenLogos,
    EtherscanTokenLogos,
    EtherscanLinks,
    EtherscanLinksTxHash,
    EtherscanLinksAddress,
    EtherscanLinksBlock,
}
