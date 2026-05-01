use std::sync::Arc;

use crate::models::network::{endpoint::NetworkEndpoint, identity::NetworkIdentity};
use alloy::{
    providers::{DynProvider, Provider, ProviderBuilder},
    transports::{RpcError as AlloyRpcError, TransportErrorKind},
};
use chrono::Utc;
use poem_openapi::{Object, Union};
use rustls::lock::Mutex;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

pub struct EthProvider {
    network_identity: NetworkIdentity,
    endpoint_identity: i32,
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

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct RpcStatusAlive {
    block_number: u64,
    network_identity: NetworkIdentity,
    timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct RpcStatusDead {
    error: String,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct RpcStatusDisabled;

#[derive(Debug, Serialize, Deserialize, Union)]
#[oai(discriminator_name = "status")]
pub enum RpcStatus {
    Alive(RpcStatusAlive),
    Dead(RpcStatusDead),
    Disabled(RpcStatusDisabled),
}

impl EthProvider {
    pub async fn start(endpoint: &NetworkEndpoint) -> Self {
        let state = Arc::new(Mutex::new(RpcState::from_disabled(
            endpoint.endpoint_disabled,
        )));

        let me = Self {
            network_identity: endpoint.network_identity.clone(),
            endpoint_identity: endpoint.endpoint_identity,
            state,
        };

        me.update(endpoint).await.unwrap();

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

        let provider = ProviderBuilder::new()
            .connect(endpoint.endpoint_url.as_str())
            .await;

        let provider = match provider {
            Ok(provider) => provider,
            Err(e) => {
                let error = RpcError::ProviderError(Arc::new(e));
                self.set_state(RpcState::Dead {
                    error: error.clone(),
                });
                return Err(error);
            }
        };

        info!("Endpoint connected, checking chain_id");

        // Verify chain id matches with the desired network
        let network_identity = match provider.get_chain_id().await {
            Ok(chain_id) => chain_id,
            Err(e) => {
                let error = RpcError::ProviderError(Arc::new(e));
                self.set_state(RpcState::Dead {
                    error: error.clone(),
                });
                return Err(error);
            }
        };
        if network_identity != self.network_identity.0 {
            self.set_state(RpcState::Dead {
                error: RpcError::ChainIdMismatch(
                    NetworkIdentity(network_identity),
                    self.network_identity.clone(),
                ),
            });
            return Err(RpcError::ChainIdMismatch(
                NetworkIdentity(network_identity),
                self.network_identity.clone(),
            ));
        }

        info!("Network identity matches, RPC started");

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
                    Err(e) => {
                        warn!("Error getting block number: {}", e);
                        return RpcStatus::Dead(RpcStatusDead {
                            error: e.to_string(),
                        });
                    }
                };

                info!("Block number: {}", block_number);

                RpcStatus::Alive(RpcStatusAlive {
                    block_number,
                    network_identity: self.network_identity.clone(),
                    timestamp: Utc::now().timestamp_millis() as u64,
                })
            }
            RpcState::Dead { error } => RpcStatus::Dead(RpcStatusDead {
                error: error.to_string(),
            }),
            RpcState::Disabled => RpcStatus::Disabled(RpcStatusDisabled),
        }
    }

    pub fn get_provider(&self) -> Option<DynProvider> {
        match self.get_state() {
            RpcState::Alive { inner } => Some(inner),
            RpcState::Dead { .. } => None,
            RpcState::Disabled => None,
        }
    }
}
