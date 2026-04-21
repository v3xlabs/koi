use poem_openapi::{Object, types::Example};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query_as};

use crate::{error::KoiError, models::network::Network, state::AppState};

#[derive(Debug, Serialize, Deserialize, FromRow, Object)]
pub struct NetworkEndpoint {
    pub endpoint_identity: String,
    pub endpoint_label: Option<String>,
    pub endpoint_type: String,
    pub endpoint_url: String,
    pub endpoint_priority: u32,
    pub endpoint_disabled: bool,
    pub network_identity: i32,
}

impl Network {
    pub async fn endpoints(state: &AppState) -> Result<Vec<NetworkEndpoint>, KoiError> {
        query_as::<_, NetworkEndpoint>("SELECT * FROM network_endpoints")
            .fetch_all(&state.database)
            .await
            .map_err(KoiError::from)
    }
}

impl NetworkEndpoint {
    pub async fn create(state: &AppState, endpoint: NetworkEndpoint) -> Result<NetworkEndpoint, KoiError> {
        query_as::<_, NetworkEndpoint>("INSERT INTO network_endpoints (endpoint_identity, endpoint_label, endpoint_type, endpoint_url, endpoint_priority, endpoint_disabled, network_identity) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(endpoint.endpoint_identity)
            .bind(endpoint.endpoint_label)
            .bind(endpoint.endpoint_type)
            .bind(endpoint.endpoint_url)
            .bind(endpoint.endpoint_priority)
            .bind(endpoint.endpoint_disabled)
            .bind(endpoint.network_identity)
            .fetch_one(&state.database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn get_by_network_id(state: &AppState, network_id: i32) -> Result<Vec<NetworkEndpoint>, KoiError> {
        query_as::<_, NetworkEndpoint>("SELECT * FROM network_endpoints WHERE network_identity = ?")
            .bind(network_id)
            .fetch_all(&state.database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn get_by_id(state: &AppState, network_id: i32, endpoint_id: String) -> Result<NetworkEndpoint, KoiError> {
        query_as::<_, NetworkEndpoint>("SELECT * FROM network_endpoints WHERE network_identity = ? AND endpoint_identity = ?")
            .bind(endpoint_id)
            .fetch_one(&state.database)
            .await
            .map_err(KoiError::from)
    }
}

impl Example for NetworkEndpoint {
    fn example() -> Self {
        Self {
            endpoint_identity: "1".to_string(),
            endpoint_label: None,
            endpoint_type: "http".to_string(),
            endpoint_url: "https://example.com".to_string(),
            endpoint_priority: 1,
            endpoint_disabled: false,
            network_identity: 1,
        }
    }
}
