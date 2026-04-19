use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use crate::{error::KoiError, state::AppState};

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct Network {
    /// evm chain id
    pub network_identity: i32,
    /// name, Ethereum Mainnet, Optimism, etc
    pub network_name: String,
    /// icon url, https://example.com/icon.png, etc
    pub network_icon_url: Option<String>,
}

impl Network {
    pub async fn all(_state: &AppState) -> Result<Vec<Network>, KoiError> {
        Ok(vec![Network {
            network_identity: 1,
            network_name: "Ethereum Mainnet".to_string(),
            network_icon_url: Some("https://example.com/icon.png".to_string()),
        }])
    }
}

impl Network {
    pub async fn get_by_id(_state: &AppState, network_id: i32) -> Result<Network, KoiError> {
        Ok(Network {
            network_identity: network_id,
            network_name: "Ethereum Mainnet".to_string(),
            network_icon_url: Some("https://example.com/icon.png".to_string()),
        })
    }
}
