use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use poem_openapi::Object;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct RpcPoolStats {
    pub network_identity: NetworkIdentity,
    pub endpoint_count: u64,
    pub alive_count: u64,
    pub dead_count: u64,
    pub disabled_count: u64,
    pub in_flight: u64,
    pub queued: u64,
    pub total_requests: u64,
    pub total_errors: u64,
    pub endpoints: Vec<RpcPoolEndpointStats>,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct RpcPoolEndpointStats {
    pub endpoint_identity: i32,
    pub status: String,
    pub in_flight: u64,
    pub queued: u64,
    pub total_requests: u64,
    pub total_errors: u64,
}

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

    pub async fn start_endpoint(
        &self,
        state: &AppState,
        endpoint: &NetworkEndpoint,
    ) -> Arc<EthProvider> {
        info!("Starting endpoint: {}", endpoint.endpoint_url);

        let rpc = Arc::new(EthProvider::start(endpoint, state.config.clone()).await);

        self.endpoints
            .lock()
            .expect("rpc pool mutex poisoned")
            .insert(endpoint.endpoint_identity, rpc.clone());

        rpc
    }

    pub fn remove_endpoint(&self, endpoint_identity: &i32) {
        self.endpoints
            .lock()
            .expect("rpc pool mutex poisoned")
            .remove(endpoint_identity);
    }

    pub fn snapshot(&self) -> RpcPoolStats {
        let endpoints = self
            .endpoints
            .lock()
            .expect("rpc pool mutex poisoned")
            .values()
            .cloned()
            .collect::<Vec<_>>();

        let endpoints = endpoints
            .iter()
            .map(|endpoint| {
                let status = match endpoint.get_state() {
                    RpcState::Alive { .. } => "Alive",
                    RpcState::Dead { .. } => "Dead",
                    RpcState::Disabled => "Disabled",
                };
                let rpc = endpoint.get_rpc_stats();

                RpcPoolEndpointStats {
                    endpoint_identity: endpoint.endpoint_identity(),
                    status: status.to_string(),
                    in_flight: rpc.in_flight,
                    queued: rpc.queued,
                    total_requests: rpc.total_requests,
                    total_errors: rpc.total_errors,
                }
            })
            .collect::<Vec<_>>();

        RpcPoolStats {
            network_identity: self.network_identity.clone(),
            endpoint_count: endpoints.len() as u64,
            alive_count: endpoints.iter().filter(|x| x.status == "Alive").count() as u64,
            dead_count: endpoints.iter().filter(|x| x.status == "Dead").count() as u64,
            disabled_count: endpoints.iter().filter(|x| x.status == "Disabled").count() as u64,
            in_flight: endpoints.iter().map(|x| x.in_flight).sum(),
            queued: endpoints.iter().map(|x| x.queued).sum(),
            total_requests: endpoints.iter().map(|x| x.total_requests).sum(),
            total_errors: endpoints.iter().map(|x| x.total_errors).sum(),
            endpoints,
        }
    }

    pub async fn get_rpc(&self, endpoint_identity: &i32, state: &AppState) -> Arc<EthProvider> {
        {
            let endpoints = self.endpoints.lock().expect("rpc pool mutex poisoned");
            if let Some(rpc) = endpoints.get(endpoint_identity) {
                return rpc.clone();
            }
        }

        let endpoint =
            NetworkEndpoint::get_by_id(&state.database, &self.network_identity, endpoint_identity)
                .await
                .unwrap();

        self.start_endpoint(state, &endpoint).await
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

        let endpoints =
            NetworkEndpoint::get_by_network_id(&state.database, &self.network_identity).await;

        if let Ok(endpoints) = endpoints {
            for endpoint in endpoints {
                if endpoint.endpoint_disabled {
                    continue;
                }
                let rpc = self.start_endpoint(state, &endpoint).await;

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
