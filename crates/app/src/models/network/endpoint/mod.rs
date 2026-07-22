use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query, query_as};
use ts_rs::TS;

use crate::{
    error::KoiError,
    models::network::{Network, identity::NetworkIdentity},
    state::DB,
};

pub mod metrics;
pub mod provider;

#[derive(Debug, Serialize, Deserialize, FromRow, TS)]
#[ts(optional_fields)]
pub struct NetworkEndpoint {
    pub endpoint_identity: i32,
    pub endpoint_label: Option<String>,
    pub endpoint_type: String,
    pub endpoint_url: String,
    pub endpoint_disabled: bool,
    pub network_identity: NetworkIdentity,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(optional_fields)]
pub struct NetworkEndpointCreate {
    pub endpoint_label: Option<String>,
    pub endpoint_type: String,
    pub endpoint_url: String,
    pub endpoint_disabled: bool,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(optional_fields)]
pub struct NetworkEndpointUpdate {
    pub endpoint_label: Option<String>,
    pub endpoint_type: Option<String>,
    pub endpoint_url: Option<String>,
    pub endpoint_disabled: Option<bool>,
}

impl Network {
    pub async fn endpoints(database: &DB) -> Result<Vec<NetworkEndpoint>, KoiError> {
        query_as::<_, NetworkEndpoint>("SELECT * FROM network_endpoints")
            .fetch_all(database)
            .await
            .map_err(KoiError::from)
    }
}

impl NetworkEndpoint {
    pub async fn create(
        database: &DB,
        network_identity: NetworkIdentity,
        endpoint: NetworkEndpointCreate,
    ) -> Result<NetworkEndpoint, KoiError> {
        query_as::<_, NetworkEndpoint>("INSERT INTO network_endpoints (endpoint_label, endpoint_type, endpoint_url, endpoint_disabled, network_identity) VALUES (?, ?, ?, ?, ?) RETURNING *")
            .bind(endpoint.endpoint_label)
            .bind(endpoint.endpoint_type)
            .bind(endpoint.endpoint_url)
            .bind(endpoint.endpoint_disabled)
            .bind(network_identity)
            .fetch_one(database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn get_by_network_id(
        database: &DB,
        network_identity: &NetworkIdentity,
    ) -> Result<Vec<NetworkEndpoint>, KoiError> {
        query_as::<_, NetworkEndpoint>("SELECT * FROM network_endpoints WHERE network_identity = ?")
            .bind(network_identity)
            .fetch_all(database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn get_by_id(
        database: &DB,
        network_identity: &NetworkIdentity,
        endpoint_identity: &i32,
    ) -> Result<NetworkEndpoint, KoiError> {
        query_as::<_, NetworkEndpoint>(
            "SELECT * FROM network_endpoints WHERE network_identity = ? AND endpoint_identity = ?",
        )
        .bind(network_identity)
        .bind(endpoint_identity)
        .fetch_one(database)
        .await
        .map_err(KoiError::from)
    }

    pub async fn delete(
        database: &DB,
        network_identity: &NetworkIdentity,
        endpoint_identity: &i32,
    ) -> Result<(), KoiError> {
        query("DELETE FROM network_endpoints WHERE network_identity = ? AND endpoint_identity = ?")
            .bind(network_identity)
            .bind(endpoint_identity)
            .execute(database)
            .await
            .map_err(KoiError::from)
            .map(|_| ())
    }

    pub async fn update(
        database: &DB,
        network_identity: &NetworkIdentity,
        endpoint_identity: &i32,
        update: NetworkEndpointUpdate,
    ) -> Result<NetworkEndpoint, KoiError> {
        query_as::<_, NetworkEndpoint>("UPDATE network_endpoints SET endpoint_label = ?, endpoint_type = ?, endpoint_url = ?, endpoint_disabled = ? WHERE network_identity = ? AND endpoint_identity = ? RETURNING *")
            .bind(update.endpoint_label)
            .bind(update.endpoint_type)
            .bind(update.endpoint_url)
            .bind(update.endpoint_disabled)
            .bind(network_identity)
            .bind(endpoint_identity)
            .fetch_one(database)
            .await
            .map_err(KoiError::from)
    }
}
