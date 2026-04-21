use poem_openapi::{Object, types::Example};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query, query_as};

use crate::{error::KoiError, state::AppState};

pub mod endpoint;

#[derive(Debug, Serialize, Deserialize, Object, FromRow)]
pub struct Network {
    /// evm chain id
    pub network_identity: i32,
    /// name, Ethereum Mainnet, Optimism, etc
    pub network_name: String,
    /// icon url, https://example.com/icon.png, etc
    pub network_icon_url: Option<String>,
}

impl Network {
    pub async fn all(state: &AppState) -> Result<Vec<Network>, KoiError> {
        query_as::<_, Network>("SELECT * FROM networks")
            .fetch_all(&state.database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn get_by_id(state: &AppState, network_id: i32) -> Result<Network, KoiError> {
        query_as::<_, Network>("SELECT * FROM networks WHERE network_identity = ?")
            .bind(network_id)
            .fetch_one(&state.database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn create(state: &AppState, network: Network) -> Result<Network, KoiError> {
        query_as::<_, Network>("INSERT INTO networks (network_identity, network_name, network_icon_url) VALUES (?, ?, ?) RETURNING *")
            .bind(network.network_identity)
            .bind(network.network_name)
            .bind(network.network_icon_url)
            .fetch_one(&state.database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn delete(state: &AppState, network_id: i32) -> Result<(), KoiError> {
        query("DELETE FROM networks WHERE network_identity = ?")
            .bind(network_id)
            .execute(&state.database)
            .await
            .map_err(KoiError::from)
            .map(|_| ())
    }

    pub fn presets() -> Vec<Network> {
        vec![
            Network {
                network_identity: 1,
                network_name: "Ethereum Mainnet".to_string(),
                network_icon_url: None,
            },
            Network {
                network_identity: 10,
                network_name: "Optimism".to_string(),
                network_icon_url: None,
            },
            Network {
                network_identity: 137,
                network_name: "Polygon".to_string(),
                network_icon_url: None,
            },
            Network {
                network_identity: 42161,
                network_name: "Arbitrum".to_string(),
                network_icon_url: None,
            },
        ]
    }
}

impl Example for Network {
    fn example() -> Self {
        Self {
            network_identity: 1,
            network_name: "Ethereum Mainnet".to_string(),
            network_icon_url: Some("https://example.com/icon.png".to_string()),
        }
    }
}
