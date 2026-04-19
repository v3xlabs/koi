use serde::{Deserialize, Serialize};
use poem_openapi::Object;

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct Network {
    /// evm chain id
    pub network_identity: i32,
    /// name, Ethereum Mainnet, Optimism, etc
    pub network_name: String,
    /// icon url, https://example.com/icon.png, etc
    pub network_icon_url: Option<String>,
}
