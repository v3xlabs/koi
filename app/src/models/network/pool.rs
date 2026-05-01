use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use sqlx::SqlitePool;
use tracing::info;

use crate::{
    error::KoiError,
    models::network::{
        endpoint::{
            NetworkEndpoint,
            provider::{EthProvider, RpcState},
        },
        identity::NetworkIdentity,
    },
    state::AppState,
};

pub struct RpcPool {
    network_identity: NetworkIdentity,
    endpoints: Mutex<HashMap<i32, Arc<EthProvider>>>,
}

impl RpcPool {
    pub fn new(network_identity: NetworkIdentity) -> Self {
        Self {
            network_identity,
            endpoints: Mutex::new(HashMap::new()),
        }
    }

    pub async fn start_endpoint(&self, endpoint: &NetworkEndpoint) -> Arc<EthProvider> {
        info!("Starting endpoint: {}", endpoint.endpoint_url);

        let rpc = Arc::new(EthProvider::start(endpoint).await);

        self.endpoints
            .lock()
            .expect("rpc pool mutex poisoned")
            .insert(endpoint.endpoint_identity, rpc.clone());

        rpc
    }

    pub async fn get_rpc(&self, endpoint_identity: &i32, db: &SqlitePool) -> Arc<EthProvider> {
        {
            let endpoints = self.endpoints.lock().expect("rpc pool mutex poisoned");
            if let Some(rpc) = endpoints.get(endpoint_identity) {
                return rpc.clone();
            }
        }

        let endpoint = NetworkEndpoint::get_by_id(db, &self.network_identity, endpoint_identity)
            .await
            .unwrap();

        self.start_endpoint(&endpoint).await
    }

    pub async fn get_first_rpc(&self, state: &AppState) -> Result<Arc<EthProvider>, KoiError> {
        {
            let endpoints = self.endpoints.lock().expect("rpc pool mutex poisoned");

            for endpoint in endpoints.values() {
                match &endpoint.get_state() {
                    RpcState::Alive { .. } => return Ok(endpoint.clone()),
                    RpcState::Dead { .. } => continue,
                    RpcState::Disabled => continue,
                }
            }
        }

        let endpoints = NetworkEndpoint::get_by_network_id(state, &self.network_identity).await;

        if let Ok(endpoints) = endpoints {
            for endpoint in endpoints {
                if endpoint.endpoint_disabled {
                    continue;
                }
                let rpc = self.start_endpoint(&endpoint).await;

                if let RpcState::Alive { inner: _ } = rpc.get_state() {
                    return Ok(rpc);
                } else {
                    continue;
                }
            }
        }

        Err(KoiError::Internal("No RPC found for network".to_string()))
    }
}
