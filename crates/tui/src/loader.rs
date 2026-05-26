use std::collections::HashMap;

use tokio::sync::mpsc;

use koi::models::{
    account::{Account, balances::AccountBalances},
    asset::{metadata::AssetMetadataDiscovery, Asset},
    network::{Network, pool::RpcPoolStats},
    tx::Tx,
};

use koi_client::ApiClient;

use super::{
    app::{App, ResourceState},
    defi::{DefiClient, DefiResult},
    settings::SettingsSnapshot,
};

#[derive(Clone)]
pub struct Loader {
    client: ApiClient,
    defi: DefiClient,
    tx: mpsc::UnboundedSender<BackgroundUpdate>,
}

pub enum BackgroundUpdate {
    Health {
        generation: u64,
        connected: bool,
        notice: Option<String>,
    },
    AccountsLoaded {
        generation: u64,
        accounts: Vec<Account>,
        notice: Option<String>,
    },
    NetworksLoaded {
        generation: u64,
        networks: Vec<Network>,
        notice: Option<String>,
    },
    AssetsLoaded {
        generation: u64,
        assets: HashMap<String, Asset>,
        notice: Option<String>,
    },
    Rpc {
        generation: u64,
        network_id: u64,
        state: ResourceState<RpcPoolStats>,
    },
    Balance {
        generation: u64,
        account_id: u64,
        state: ResourceState<AccountBalances>,
    },
    Defi {
        generation: u64,
        account_id: u64,
        state: ResourceState<DefiResult>,
    },
    Transactions {
        generation: u64,
        account_id: u64,
        state: ResourceState<Vec<Tx>>,
    },
    Settings {
        generation: u64,
        state: ResourceState<SettingsSnapshot>,
    },
    EndpointNextId {
        generation: u64,
        network_id: u64,
        next_id: i32,
    },
    AssetMetadata {
        generation: u64,
        identity: String,
        result: Result<AssetMetadataDiscovery, String>,
    },
    Notice {
        generation: u64,
        notice: String,
    },
}

impl Loader {
    pub fn new(client: ApiClient, tx: mpsc::UnboundedSender<BackgroundUpdate>) -> Self {
        Self {
            client,
            defi: DefiClient::new(),
            tx,
        }
    }

    pub fn spawn_refresh_all(&self, generation: u64) {
        let client = self.client.clone();
        let tx = self.tx.clone();

        tokio::spawn(async move {
            match client.health().await {
                Ok(()) => {
                    let _ = tx.send(BackgroundUpdate::Health {
                        generation,
                        connected: true,
                        notice: None,
                    });
                }
                Err(error) => {
                    let _ = tx.send(BackgroundUpdate::Health {
                        generation,
                        connected: false,
                        notice: Some(format!("Cannot reach server: {error:#}")),
                    });
                    return;
                }
            }

            let accounts = match client.accounts().await {
                Ok(accounts) => accounts,
                Err(error) => {
                    let _ = tx.send(BackgroundUpdate::AccountsLoaded {
                        generation,
                        accounts: Vec::new(),
                        notice: Some(format!("Accounts: {error:#}")),
                    });
                    Vec::new()
                }
            };
            let account_ids: Vec<u64> = accounts.iter().map(|account| account.account_identity.0).collect();
            let _ = tx.send(BackgroundUpdate::AccountsLoaded {
                generation,
                accounts,
                notice: None,
            });

            let networks = match client.networks().await {
                Ok(networks) => networks,
                Err(error) => {
                    let _ = tx.send(BackgroundUpdate::NetworksLoaded {
                        generation,
                        networks: Vec::new(),
                        notice: Some(format!("Networks: {error:#}")),
                    });
                    Vec::new()
                }
            };
            let network_ids: Vec<u64> = networks.iter().map(|network| network.network_identity.0).collect();
            let _ = tx.send(BackgroundUpdate::NetworksLoaded {
                generation,
                networks,
                notice: None,
            });

            let assets = match client.assets().await {
                Ok(assets) => assets,
                Err(error) => {
                    let _ = tx.send(BackgroundUpdate::AssetsLoaded {
                        generation,
                        assets: HashMap::new(),
                        notice: Some(format!("Assets: {error:#}")),
                    });
                    HashMap::new()
                }
            };
            let _ = tx.send(BackgroundUpdate::AssetsLoaded {
                generation,
                assets,
                notice: None,
            });

            for network_id in &network_ids {
                spawn_rpc_fetch(client.clone(), tx.clone(), generation, *network_id);
            }

            for account_id in account_ids {
                spawn_balance_fetch(client.clone(), tx.clone(), generation, account_id);
            }

            spawn_settings_fetch(client.clone(), tx.clone(), generation, network_ids);
        });
    }

    pub fn spawn_balance(&self, generation: u64, account_id: u64) {
        spawn_balance_fetch(self.client.clone(), self.tx.clone(), generation, account_id);
    }

    pub fn spawn_defi(&self, generation: u64, account_id: u64, holder: String) {
        spawn_defi_fetch(
            self.defi.clone(),
            self.tx.clone(),
            generation,
            account_id,
            holder,
        );
    }

    pub fn spawn_transactions(&self, generation: u64, account_id: u64) {
        spawn_transactions_fetch(self.client.clone(), self.tx.clone(), generation, account_id);
    }

    pub fn spawn_settings(&self, generation: u64, network_ids: Vec<u64>) {
        spawn_settings_fetch(self.client.clone(), self.tx.clone(), generation, network_ids);
    }

    pub fn delete_network_endpoint(
        &self,
        generation: u64,
        network_id: u64,
        endpoint_id: i32,
        network_ids: Vec<u64>,
    ) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let notice = match client
                .delete_network_endpoint(network_id, endpoint_id)
                .await
            {
                Ok(()) => format!("Deleted endpoint #{endpoint_id} on network {network_id}"),
                Err(error) => format!("Delete endpoint failed: {error:#}"),
            };
            let _ = tx.send(BackgroundUpdate::Notice { generation, notice });
            spawn_settings_fetch(client, tx, generation, network_ids);
        });
    }

    pub fn delete_asset(&self, generation: u64, asset_identity: String, network_ids: Vec<u64>) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let notice = match client.delete_asset(&asset_identity).await {
                Ok(()) => format!("Deleted asset {asset_identity}"),
                Err(error) => format!("Delete asset failed: {error:#}"),
            };
            let _ = tx.send(BackgroundUpdate::Notice { generation, notice });
            spawn_settings_fetch(client, tx, generation, network_ids);
        });
    }

    pub fn set_vendor(
        &self,
        generation: u64,
        flag: String,
        enabled: bool,
        network_ids: Vec<u64>,
    ) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let notice = match client.set_vendor(&flag, enabled).await {
                Ok(()) => format!(
                    "{} vendor {flag}",
                    if enabled { "Enabled" } else { "Disabled" }
                ),
                Err(error) => format!("Vendor update failed: {error:#}"),
            };
            let _ = tx.send(BackgroundUpdate::Notice { generation, notice });
            spawn_settings_fetch(client, tx, generation, network_ids);
        });
    }

    pub fn fetch_asset_metadata(&self, generation: u64, identity: String) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let result = match client.asset_metadata_discovery(&identity).await {
                Ok(discovery) => Ok(discovery),
                Err(error) => Err(error.to_string()),
            };
            let _ = tx.send(BackgroundUpdate::AssetMetadata {
                generation,
                identity,
                result,
            });
        });
    }

    pub fn fetch_endpoint_next_id(&self, generation: u64, network_id: u64) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let next_id = match client.network_endpoint_next_id(network_id).await {
                Ok(next_id) => next_id,
                Err(error) => {
                    let _ = tx.send(BackgroundUpdate::Notice {
                        generation,
                        notice: format!("Could not fetch endpoint id: {error:#}"),
                    });
                    1
                }
            };
            let _ = tx.send(BackgroundUpdate::EndpointNextId {
                generation,
                network_id,
                next_id,
            });
        });
    }

    pub fn create_asset(&self, generation: u64, asset: koi::models::asset::Asset, network_ids: Vec<u64>) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let notice = match client.create_asset(&asset).await {
                Ok(created) => format!("Created asset {}", created.asset_identity),
                Err(error) => format!("Create asset failed: {error:#}"),
            };
            let _ = tx.send(BackgroundUpdate::Notice { generation, notice });
            if let Ok(assets) = client.assets().await {
                let _ = tx.send(BackgroundUpdate::AssetsLoaded {
                    generation,
                    assets,
                    notice: None,
                });
            }
            spawn_settings_fetch(client, tx, generation, network_ids);
        });
    }

    pub fn create_network(&self, generation: u64, network: koi::models::network::Network, network_ids: Vec<u64>) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let notice = match client.create_network(&network).await {
                Ok(created) => format!("Created network {}", created.network_name),
                Err(error) => format!("Create network failed: {error:#}"),
            };
            let _ = tx.send(BackgroundUpdate::Notice { generation, notice });
            if let Ok(networks) = client.networks().await {
                let _ = tx.send(BackgroundUpdate::NetworksLoaded {
                    generation,
                    networks,
                    notice: None,
                });
            }
            let refreshed_ids = client
                .networks()
                .await
                .map(|networks| {
                    networks
                        .iter()
                        .map(|network| network.network_identity.0)
                        .collect::<Vec<_>>()
                })
                .unwrap_or(network_ids);
            spawn_settings_fetch(client, tx, generation, refreshed_ids);
        });
    }

    pub fn create_network_endpoint(
        &self,
        generation: u64,
        endpoint: koi::models::network::endpoint::NetworkEndpoint,
        network_ids: Vec<u64>,
    ) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        let network_id = endpoint.network_identity.0;
        tokio::spawn(async move {
            let notice = match client.create_network_endpoint(network_id, &endpoint).await {
                Ok(created) => format!("Created endpoint #{}", created.endpoint_identity),
                Err(error) => format!("Create endpoint failed: {error:#}"),
            };
            let _ = tx.send(BackgroundUpdate::Notice { generation, notice });
            spawn_settings_fetch(client, tx, generation, network_ids);
        });
    }
}

fn spawn_rpc_fetch(
    client: ApiClient,
    tx: mpsc::UnboundedSender<BackgroundUpdate>,
    generation: u64,
    network_id: u64,
) {
    let _ = tx.send(BackgroundUpdate::Rpc {
        generation,
        network_id,
        state: ResourceState::Loading,
    });

    tokio::spawn(async move {
        let state = match client.network_rpc_stats(network_id).await {
            Ok(stats) => ResourceState::Ready(stats),
            Err(error) => ResourceState::Error(error.to_string()),
        };
        let _ = tx.send(BackgroundUpdate::Rpc {
            generation,
            network_id,
            state,
        });
    });
}

fn spawn_balance_fetch(
    client: ApiClient,
    tx: mpsc::UnboundedSender<BackgroundUpdate>,
    generation: u64,
    account_id: u64,
) {
    let _ = tx.send(BackgroundUpdate::Balance {
        generation,
        account_id,
        state: ResourceState::Loading,
    });

    tokio::spawn(async move {
        let state = match client.account_balances(account_id).await {
            Ok(balances) => ResourceState::Ready(balances),
            Err(error) => ResourceState::Error(error.to_string()),
        };
        let _ = tx.send(BackgroundUpdate::Balance {
            generation,
            account_id,
            state,
        });
    });
}

fn spawn_defi_fetch(
    client: DefiClient,
    tx: mpsc::UnboundedSender<BackgroundUpdate>,
    generation: u64,
    account_id: u64,
    holder: String,
) {
    let _ = tx.send(BackgroundUpdate::Defi {
        generation,
        account_id,
        state: ResourceState::Loading,
    });

    tokio::spawn(async move {
        let result = client.positions(&holder).await;
        let _ = tx.send(BackgroundUpdate::Defi {
            generation,
            account_id,
            state: ResourceState::Ready(result),
        });
    });
}

fn spawn_transactions_fetch(
    client: ApiClient,
    tx: mpsc::UnboundedSender<BackgroundUpdate>,
    generation: u64,
    account_id: u64,
) {
    let _ = tx.send(BackgroundUpdate::Transactions {
        generation,
        account_id,
        state: ResourceState::Loading,
    });

    tokio::spawn(async move {
        let state = match client.account_transactions(account_id).await {
            Ok(transactions) => ResourceState::Ready(transactions),
            Err(error) => ResourceState::Error(error.to_string()),
        };
        let _ = tx.send(BackgroundUpdate::Transactions {
            generation,
            account_id,
            state,
        });
    });
}

fn spawn_settings_fetch(
    client: ApiClient,
    tx: mpsc::UnboundedSender<BackgroundUpdate>,
    generation: u64,
    network_ids: Vec<u64>,
) {
    let _ = tx.send(BackgroundUpdate::Settings {
        generation,
        state: ResourceState::Loading,
    });

    tokio::spawn(async move {
        let mut endpoints = HashMap::new();
        for network_id in network_ids {
            match client.network_endpoints(network_id).await {
                Ok(network_endpoints) => {
                    endpoints.insert(network_id, network_endpoints);
                }
                Err(error) => {
                    let _ = tx.send(BackgroundUpdate::Notice {
                        generation,
                        notice: format!("Endpoints for {network_id}: {error:#}"),
                    });
                }
            }
        }

        let quoters = match client.quoters().await {
            Ok(quoters) => quoters,
            Err(error) => {
                let _ = tx.send(BackgroundUpdate::Notice {
                    generation,
                    notice: format!("Price feeds: {error:#}"),
                });
                Vec::new()
            }
        };

        let all_vendors = match client.all_vendors().await {
            Ok(vendors) => vendors,
            Err(error) => {
                let _ = tx.send(BackgroundUpdate::Notice {
                    generation,
                    notice: format!("Vendors: {error:#}"),
                });
                Vec::new()
            }
        };

        let enabled_vendors = match client.vendors().await {
            Ok(vendors) => vendors.into_iter().map(|vendor| vendor.to_string()).collect(),
            Err(error) => {
                let _ = tx.send(BackgroundUpdate::Notice {
                    generation,
                    notice: format!("Enabled vendors: {error:#}"),
                });
                std::collections::HashSet::new()
            }
        };

        let _ = tx.send(BackgroundUpdate::Settings {
            generation,
            state: ResourceState::Ready(SettingsSnapshot {
                endpoints,
                quoters,
                all_vendors,
                enabled_vendors,
            }),
        });
    });
}

pub fn prepare_refresh_all(app: &mut App) -> u64 {
    app.begin_refresh()
}
