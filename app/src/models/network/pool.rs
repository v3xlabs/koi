use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use sqlx::SqlitePool;
use tracing::info;

use crate::models::network::{
    endpoint::{
        NetworkEndpoint,
        provider::{EthProvider, RpcState},
    },
    identity::NetworkIdentity,
};

pub struct RpcPool {
    network_identity: NetworkIdentity,
    endpoints: Mutex<HashMap<String, Arc<EthProvider>>>,
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
            .insert(endpoint.endpoint_identity.clone(), rpc.clone());

        rpc
    }

    pub async fn get_rpc(&self, endpoint_identity: &str, db: &SqlitePool) -> Arc<EthProvider> {
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

    pub fn get_first_rpc(&self) -> Option<Arc<EthProvider>> {
        let endpoints = self.endpoints.lock().expect("rpc pool mutex poisoned");

        for endpoint in endpoints.values() {
            match &endpoint.get_state() {
                RpcState::Alive { .. } => return Some(endpoint.clone()),
                RpcState::Dead { .. } => continue,
                RpcState::Disabled => continue,
            }
        }

        None
    }
}
