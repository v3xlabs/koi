use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query, query_as};

use crate::{error::KoiError, models::network::identity::NetworkIdentity, state::DB};

pub mod endpoint;
pub mod identity;
pub mod manager;
pub mod metadata;
pub mod pool;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Network {
    /// evm chain id
    pub network_identity: NetworkIdentity,
    /// name, Ethereum Mainnet, Optimism, etc
    pub network_name: String,
    /// icon url, https://example.com/icon.png, etc
    pub network_icon_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkUpdate {
    pub network_name: Option<String>,
    pub network_icon_url: Option<String>,
}

impl Network {
    pub async fn all(database: &DB) -> Result<Vec<Network>, KoiError> {
        query_as::<_, Network>("SELECT * FROM networks")
            .fetch_all(database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn get_by_id(
        database: &DB,
        network_identity: &NetworkIdentity,
    ) -> Result<Network, KoiError> {
        query_as::<_, Network>("SELECT * FROM networks WHERE network_identity = ?")
            .bind(network_identity)
            .fetch_one(database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn create(database: &DB, network: Network) -> Result<Network, KoiError> {
        query_as::<_, Network>("INSERT INTO networks (network_identity, network_name, network_icon_url) VALUES (?, ?, ?) RETURNING *")
            .bind(network.network_identity)
            .bind(network.network_name)
            .bind(network.network_icon_url)
            .fetch_one(database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn update(
        database: &DB,
        network_identity: &NetworkIdentity,
        network: NetworkUpdate,
    ) -> Result<Network, KoiError> {
        let original = Self::get_by_id(database, network_identity).await?;

        let network_name = network.network_name.unwrap_or(original.network_name);
        let network_icon_url = match network.network_icon_url {
            Some(url) if url.trim().is_empty() => None,
            Some(url) => Some(url),
            None => original.network_icon_url,
        };

        query_as::<_, Network>("UPDATE networks SET network_name = ?, network_icon_url = ? WHERE network_identity = ? RETURNING *")
            .bind(network_name)
            .bind(network_icon_url)
            .bind(network_identity)
            .fetch_one(database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn delete(database: &DB, network_identity: &NetworkIdentity) -> Result<(), KoiError> {
        query("DELETE FROM networks WHERE network_identity = ?")
            .bind(network_identity)
            .execute(database)
            .await
            .map_err(KoiError::from)
            .map(|_| ())
    }

    pub fn presets() -> Vec<Network> {
        vec![
            Network {
                network_identity: NetworkIdentity(1),
                network_name: "Ethereum Mainnet".to_string(),
                network_icon_url: Some("data:image/webp;base64,UklGRqIBAABXRUJQVlA4TJYBAAAvG8AGAIVka9sMSdJqVubKtm2bKxs7rWzbtm0jM+NnRGRhPJc0f9clRPR/Avj/nj1qGpB9EABicxZBWH2K6DJUrYMrOcohICs7UEUbR4DS8wc/QKInILGpIwin1npyqAoYbT7N5cvuwZfLkpFXrqNghFt7+8xH74m8ZDZg+HDg0PCuN78WKZEAtpCEM8ePDe0565OX4PJoSwovDp7b321op8MFpUL21abu9P6TZ8b37zaWOH4OuS3ePnVl6/z2XUZ02vTnS85i0/N7lk7qV69l227jvkARwHZxzoRu3Ya3qt+oZf3VvxIs2Xb26TppWPemTZs3qL+af3wiW/5war/BI1s0rO32CFW92N49+rGzb98G9deVDp38kQSwwe2TaTq13xM3byGogrfp12MnXhT80SE9P/wDQrSRvDyy7/jCQd2PlZ/nUbzNBXfp/KkhA5Yr/gj4BW25FN4f3j5xxMP40X2OZWdLg9CbQ6M25r7EpSR4G6F6PD2lIJyJYgCbzyC6F6+RK6okgdNMK0MiyCFQpdihV2cD".to_string()),
            },
            Network {
                network_identity: NetworkIdentity(10),
                network_name: "Optimism".to_string(),
                // TODO: replace these with either optimized base64 import macros
                network_icon_url: Some("https://icons.llamao.fi/icons/chains/rsz_optimism.jpg".to_string()),
            },
            Network {
                network_identity: NetworkIdentity(11155111),
                network_name: "Sepolia Testnet".to_string(),
                // TODO: replace these with either optimized base64 import macros
                network_icon_url: Some("https://chainlist.org/unknown-logo.png".to_string()),
            },
            Network {
                network_identity: NetworkIdentity(137),
                network_name: "Polygon".to_string(),
                network_icon_url: Some("https://icons.llamao.fi/icons/chains/rsz_polygon.jpg".to_string()),
            },
            Network {
                network_identity: NetworkIdentity(42161),
                network_name: "Arbitrum".to_string(),
                network_icon_url: Some("https://icons.llamao.fi/icons/chains/rsz_arbitrum.jpg".to_string()),
            },
        ]
    }
}
