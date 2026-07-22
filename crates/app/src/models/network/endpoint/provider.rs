use std::sync::Arc;

use crate::{
    config::Configuration,
    models::network::{
        endpoint::{
            NetworkEndpoint,
            metrics::{RpcEndpointStats, RpcMetrics, RpcMetricsLayer},
        },
        identity::NetworkIdentity,
    },
};
use alloy::{
    providers::{DynProvider, Provider, ProviderBuilder},
    rpc::client::ClientBuilder,
    transports::{
        RpcError as AlloyRpcError, TransportErrorKind,
        layers::{RetryBackoffLayer, ThrottleLayer},
    },
};
use chrono::Utc;
use rustls::lock::Mutex;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use ts_rs::TS;

pub struct EthProvider {
    network_identity: NetworkIdentity,
    endpoint_identity: i32,
    config: Configuration,
    metrics: RpcMetrics,
    state: Arc<Mutex<RpcState>>,
}

#[derive(Clone)]
pub enum RpcState {
    Alive { inner: DynProvider },
    Dead { error: RpcError },
    Disabled,
}

impl RpcState {
    pub fn from_disabled(is_disabled: bool) -> Self {
        match is_disabled {
            true => RpcState::Disabled,
            false => RpcState::Dead {
                error: RpcError::Starting("Starting...".to_string()),
            },
        }
    }

    pub fn from_alive(provider: DynProvider) -> Self {
        RpcState::Alive { inner: provider }
    }

    pub fn from_dead(error: RpcError) -> Self {
        RpcState::Dead { error }
    }
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum RpcError {
    #[error("Provider error: {0}")]
    ProviderError(#[from] Arc<AlloyRpcError<TransportErrorKind>>),
    #[error("Chain id mismatch: {0} != {1}")]
    ChainIdMismatch(NetworkIdentity, NetworkIdentity),
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Starting: {0}")]
    Starting(String),
    #[error("Network identity is not an EVM network")]
    NetworkIdentityNotEvm,
}

#[derive(Debug, Serialize, Deserialize, TS)]
pub struct RpcStatusAlive {
    block_number: u64,
    network_identity: NetworkIdentity,
    timestamp: u64,
    rpc: RpcEndpointStats,
}

#[derive(Debug, Serialize, Deserialize, TS)]
pub struct RpcStatusDead {
    error: String,
    rpc: RpcEndpointStats,
}

#[derive(Debug, Serialize, Deserialize, TS)]
pub struct RpcStatusDisabled {
    rpc: RpcEndpointStats,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(tag = "status")]
pub enum RpcStatus {
    Alive(RpcStatusAlive),
    Dead(RpcStatusDead),
    Disabled(RpcStatusDisabled),
}

impl EthProvider {
    pub async fn start(endpoint: &NetworkEndpoint, config: Configuration) -> Self {
        let state = Arc::new(Mutex::new(RpcState::from_disabled(
            endpoint.endpoint_disabled,
        )));
        let metrics = RpcMetrics::new(config.rpc_recent_sample_limit);

        let me = Self {
            network_identity: endpoint.network_identity.clone(),
            endpoint_identity: endpoint.endpoint_identity,
            config,
            metrics,
            state,
        };

        if let Err(error) = me.update(endpoint).await {
            warn!("Failed to start RPC endpoint: {}", error);
        }

        me
    }

    fn set_state(&self, new_state: RpcState) {
        let mut state = self.state.lock().expect("rpc state mutex poisoned");
        *state = new_state;
    }

    pub fn get_state(&self) -> RpcState {
        let state = self.state.lock().expect("rpc state mutex poisoned");
        state.clone()
    }

    pub async fn update(&self, endpoint: &NetworkEndpoint) -> Result<(), RpcError> {
        if endpoint.endpoint_disabled {
            self.set_state(RpcState::Disabled);
            return Ok(());
        }

        self.metrics.note_connection_attempt();
        let client = ClientBuilder::default()
            .layer(RpcMetricsLayer::new(
                self.metrics.clone(),
                self.config.rpc_max_in_flight_per_endpoint,
            ))
            .layer(ThrottleLayer::new(
                self.config.rpc_requests_per_second.max(1),
            ))
            .layer(RetryBackoffLayer::new(
                self.config.rpc_rate_limit_retries,
                300,
                u64::from(self.config.rpc_requests_per_second.max(1)) * 20,
            ))
            .connect(endpoint.endpoint_url.as_str())
            .await;

        let client = match client {
            Ok(client) => client,
            Err(error) => {
                let error = RpcError::ProviderError(Arc::new(error));
                self.metrics.note_connection_failure(error.to_string());
                self.set_state(RpcState::Dead {
                    error: error.clone(),
                });
                return Err(error);
            }
        };
        let provider = ProviderBuilder::new().connect_client(client);

        info!("Endpoint connected, checking chain_id");

        let network_identity = match provider.get_chain_id().await {
            Ok(chain_id) => chain_id,
            Err(error) => {
                let error = RpcError::ProviderError(Arc::new(error));
                self.metrics.note_connection_failure(error.to_string());
                self.set_state(RpcState::Dead {
                    error: error.clone(),
                });
                return Err(error);
            }
        };
        if network_identity != self.network_identity.0 {
            let error = RpcError::ChainIdMismatch(
                NetworkIdentity(network_identity),
                self.network_identity.clone(),
            );
            self.metrics.note_connection_failure(error.to_string());
            self.set_state(RpcState::Dead {
                error: error.clone(),
            });
            return Err(error);
        }

        info!("Network identity matches, RPC started");
        self.metrics.note_connection_success();

        self.set_state(RpcState::Alive {
            inner: DynProvider::new(provider),
        });
        Ok(())
    }

    pub async fn get_status(&self) -> RpcStatus {
        info!("Getting status of RPC");
        let current_state = self.get_state();

        match current_state {
            RpcState::Alive { inner } => {
                let block_number = match inner.get_block_number().await {
                    Ok(block_number) => block_number,
                    Err(error) => {
                        warn!("Error getting block number: {}", error);
                        return RpcStatus::Dead(RpcStatusDead {
                            error: error.to_string(),
                            rpc: self.metrics.snapshot(),
                        });
                    }
                };

                info!("Block number: {}", block_number);

                RpcStatus::Alive(RpcStatusAlive {
                    block_number,
                    network_identity: self.network_identity.clone(),
                    timestamp: Utc::now().timestamp_millis() as u64,
                    rpc: self.metrics.snapshot(),
                })
            }
            RpcState::Dead { error } => RpcStatus::Dead(RpcStatusDead {
                error: error.to_string(),
                rpc: self.metrics.snapshot(),
            }),
            RpcState::Disabled => RpcStatus::Disabled(RpcStatusDisabled {
                rpc: self.metrics.snapshot(),
            }),
        }
    }

    pub fn get_rpc_stats(&self) -> RpcEndpointStats {
        self.metrics.snapshot()
    }

    pub fn endpoint_identity(&self) -> i32 {
        self.endpoint_identity
    }

    pub fn get_provider(&self) -> Option<DynProvider> {
        match self.get_state() {
            RpcState::Alive { inner } => Some(inner),
            RpcState::Dead { .. } => None,
            RpcState::Disabled => None,
        }
    }
}
