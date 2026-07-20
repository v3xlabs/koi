use std::collections::HashMap;

use tokio::sync::mpsc;

use koi::{
    models::{
        account::{
            Account,
            balances::AccountBalances,
            group::AccountGroup,
            identity::AccountIdentity,
            metadata::{EOAWallet, SafeWallet, ViewWallet, WalletType},
            rpc::{
                AccountAssetParams, AccountCreateParams, AccountParams, AccountUpdateParams,
                DeriveMnemonicInput, DeriveMnemonicParams, DeriveMnemonicResult,
                DerivePrivateKeyParams,
            },
        },
        asset::{Asset, metadata::AssetMetadataDiscovery},
        network::{Network, identity::NetworkIdentity, pool::RpcPoolStats},
        tx::Tx,
    },
    rpc::EmptyParams,
};

use koi_client::ApiClient;

use super::{
    app::{App, ResourceState},
    defi::{DefiClient, DefiResult},
    form::NewAccountWallet,
    settings::SettingsSnapshot,
};

type AccountCreateMethod = koi::models::account::rpc::AccountCreate;
type AccountNextIdentityMethod = koi::models::account::rpc::AccountNextIdentity;
type AccountUpdateMethod = koi::models::account::rpc::AccountUpdate;
type AccountDeleteMethod = koi::models::account::rpc::AccountDelete;
type AccountAssetListMethod = koi::models::account::rpc::AccountAssetList;
type AccountAssetAddMethod = koi::models::account::rpc::AccountAssetAdd;
type AccountAssetRemoveMethod = koi::models::account::rpc::AccountAssetRemove;
type AssetIconMethod = koi::models::asset::rpc::AssetIcon;
type DerivationDefaultPathMethod = koi::models::account::rpc::AccountDerivationDefaultPath;
type DerivationFromMnemonicMethod = koi::models::account::rpc::AccountDerivationFromMnemonic;
type DerivationFromPrivateKeyMethod = koi::models::account::rpc::AccountDerivationFromPrivateKey;

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
    LayoutLoaded {
        generation: u64,
        groups: Vec<AccountGroup>,
        accounts: Vec<Account>,
        notice: Option<String>,
    },
    LayoutCommitted {
        generation: u64,
        groups: Vec<AccountGroup>,
        accounts: Vec<Account>,
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
        display_currency: String,
        state: ResourceState<AccountBalances>,
        // true while a fresher fetch for this account is still in flight
        refreshing: bool,
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
        next_id: i32,
    },
    AssetMetadata {
        generation: u64,
        identity: String,
        result: Result<AssetMetadataDiscovery, String>,
    },
    AssetQuote {
        generation: u64,
        identity: String,
        display_currency: String,
        state: ResourceState<String>,
    },
    AssetIcon {
        generation: u64,
        identity: String,
        icon: Option<koi::models::asset::AssetIconData>,
    },
    NetworkPresets {
        generation: u64,
        presets: Vec<Network>,
    },
    DerivationPath {
        generation: u64,
        path: String,
    },
    GeneratedMnemonic {
        generation: u64,
        mnemonic: String,
    },
    EnsResolved {
        generation: u64,
        name: String,
        address: Option<String>,
    },
    EnsReversed {
        generation: u64,
        address: String,
        name: Option<String>,
    },
    DerivedAddresses {
        generation: u64,
        name: String,
        result: Result<Vec<DeriveMnemonicResult>, String>,
    },
    AccountAssets {
        generation: u64,
        account_id: u64,
        unlink: bool,
        identities: Vec<String>,
    },
    QuoterDiscovered {
        generation: u64,
        token_a: String,
        token_b: Option<String>,
        result: Result<koi::models::quoter::discover::QuoterDiscoveryResponse, String>,
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

    pub fn spawn_refresh_all(
        &self,
        generation: u64,
        fresh_balances: bool,
        display_currency: String,
    ) {
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

            let layout = match client.account_layout().await {
                Ok(layout) => layout,
                Err(error) => {
                    let _ = tx.send(BackgroundUpdate::LayoutLoaded {
                        generation,
                        groups: Vec::new(),
                        accounts: Vec::new(),
                        notice: Some(format!("Accounts: {error:#}")),
                    });
                    koi::models::account::layout::AccountLayout {
                        groups: Vec::new(),
                        accounts: Vec::new(),
                    }
                }
            };
            let account_ids: Vec<u64> = layout
                .accounts
                .iter()
                .map(|account| account.account_identity.0)
                .collect();
            let _ = tx.send(BackgroundUpdate::LayoutLoaded {
                generation,
                groups: layout.groups,
                accounts: layout.accounts,
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
            let network_ids: Vec<u64> = networks
                .iter()
                .map(|network| network.network_identity.0)
                .collect();
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
            let asset_ids: Vec<_> = assets
                .values()
                .filter(|asset| asset.asset_icon_url.is_some())
                .map(|asset| asset.asset_identity.clone())
                .collect();
            let _ = tx.send(BackgroundUpdate::AssetsLoaded {
                generation,
                assets,
                notice: None,
            });
            for asset_identity in asset_ids {
                spawn_asset_icon_fetch(client.clone(), tx.clone(), generation, asset_identity);
            }

            for network_id in &network_ids {
                spawn_rpc_fetch(client.clone(), tx.clone(), generation, *network_id);
            }

            for account_id in account_ids {
                spawn_balance_fetch(
                    client.clone(),
                    tx.clone(),
                    generation,
                    account_id,
                    display_currency.clone(),
                    fresh_balances,
                );
            }

            spawn_settings_fetch(client.clone(), tx.clone(), generation, network_ids);
        });
    }

    pub fn spawn_balance(&self, generation: u64, account_id: u64, display_currency: String) {
        spawn_balance_fetch(
            self.client.clone(),
            self.tx.clone(),
            generation,
            account_id,
            display_currency,
            true,
        );
    }

    pub fn spawn_balance_swap(
        &self,
        generation: u64,
        account_ids: Vec<u64>,
        display_currency: String,
    ) {
        for account_id in account_ids {
            spawn_balance_swap_fetch(
                self.client.clone(),
                self.tx.clone(),
                generation,
                account_id,
                display_currency.clone(),
            );
        }
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
        spawn_settings_fetch(
            self.client.clone(),
            self.tx.clone(),
            generation,
            network_ids,
        );
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

    pub fn set_vendor(&self, generation: u64, flag: String, enabled: bool, network_ids: Vec<u64>) {
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

    pub fn commit_layout(
        &self,
        generation: u64,
        update: koi::models::account::layout::AccountLayoutUpdate,
    ) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            match client.update_account_layout(&update).await {
                Ok(layout) => {
                    let _ = tx.send(BackgroundUpdate::LayoutCommitted {
                        generation,
                        groups: layout.groups,
                        accounts: layout.accounts,
                    });
                }
                Err(error) => {
                    let _ = tx.send(BackgroundUpdate::Notice {
                        generation,
                        notice: format!("Save layout failed: {error:#}"),
                    });
                    spawn_layout_fetch(client, tx, generation);
                }
            }
        });
    }

    pub fn create_group(&self, generation: u64, name: String) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let notice = match client.create_account_group(&name).await {
                Ok(group) => format!("Created group {}", group.name),
                Err(error) => format!("Create group failed: {error:#}"),
            };
            let _ = tx.send(BackgroundUpdate::Notice { generation, notice });
            spawn_layout_fetch(client, tx, generation);
        });
    }

    pub fn rename_group(&self, generation: u64, group_id: u64, name: String) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let notice = match client.rename_account_group(group_id, &name).await {
                Ok(group) => format!("Renamed group to {}", group.name),
                Err(error) => format!("Rename group failed: {error:#}"),
            };
            let _ = tx.send(BackgroundUpdate::Notice { generation, notice });
            spawn_layout_fetch(client, tx, generation);
        });
    }

    pub fn delete_group(&self, generation: u64, group_id: u64) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let notice = match client.delete_account_group(group_id).await {
                Ok(()) => "Deleted group; its accounts are now ungrouped".to_string(),
                Err(error) => format!("Delete group failed: {error:#}"),
            };
            let _ = tx.send(BackgroundUpdate::Notice { generation, notice });
            spawn_layout_fetch(client, tx, generation);
        });
    }

    pub fn spawn_asset_quotes(
        &self,
        generation: u64,
        identities: Vec<String>,
        display_currency: String,
    ) {
        for identity in identities {
            let client = self.client.clone();
            let tx = self.tx.clone();
            let display_currency = display_currency.clone();
            tokio::spawn(async move {
                let state = match client.asset_quote(&identity, &display_currency).await {
                    Ok(quote) => ResourceState::Ready(quote),
                    Err(error) => ResourceState::Error(error.to_string()),
                };
                let _ = tx.send(BackgroundUpdate::AssetQuote {
                    generation,
                    identity,
                    display_currency,
                    state,
                });
            });
        }
    }

    pub fn fetch_derivation_path(&self, generation: u64) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            if let Ok(path) = client
                .call_typed::<DerivationDefaultPathMethod>(EmptyParams::default())
                .await
            {
                let _ = tx.send(BackgroundUpdate::DerivationPath { generation, path });
            }
        });
    }

    pub fn generate_mnemonic(&self, generation: u64) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            match client
                .call_typed::<koi::models::account::rpc::AccountMnemonicGenerate>(
                    EmptyParams::default(),
                )
                .await
            {
                Ok(mnemonic) => {
                    let _ = tx.send(BackgroundUpdate::GeneratedMnemonic {
                        generation,
                        mnemonic,
                    });
                }
                Err(error) => {
                    let _ = tx.send(BackgroundUpdate::Notice {
                        generation,
                        notice: format!("Mnemonic generation failed: {error:#}"),
                    });
                }
            }
            if let Ok(path) = client
                .call_typed::<DerivationDefaultPathMethod>(EmptyParams::default())
                .await
            {
                let _ = tx.send(BackgroundUpdate::DerivationPath { generation, path });
            }
        });
    }

    pub fn resolve_ens(&self, generation: u64, name: String) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let address = client
                .call_typed::<koi::models::ens::EnsResolve>(koi::models::ens::EnsResolveParams {
                    name: name.clone(),
                })
                .await
                .ok()
                .flatten();
            let _ = tx.send(BackgroundUpdate::EnsResolved {
                generation,
                name,
                address,
            });
        });
    }

    pub fn reverse_ens(&self, generation: u64, address: String) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let name = client
                .call_typed::<koi::models::ens::EnsReverse>(koi::models::ens::EnsReverseParams {
                    address: address.clone(),
                })
                .await
                .ok()
                .flatten();
            let _ = tx.send(BackgroundUpdate::EnsReversed {
                generation,
                address,
                name,
            });
        });
    }

    pub fn derive_addresses(
        &self,
        generation: u64,
        name: String,
        mnemonic: String,
        paths: Vec<String>,
    ) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let result = client
                .call_typed::<DerivationFromMnemonicMethod>(DeriveMnemonicParams {
                    input: DeriveMnemonicInput { mnemonic, paths },
                })
                .await
                .map_err(|error| error.to_string());
            let _ = tx.send(BackgroundUpdate::DerivedAddresses {
                generation,
                name,
                result,
            });
        });
    }

    pub fn create_account(
        &self,
        generation: u64,
        name: String,
        wallet: NewAccountWallet,
        networks: Vec<u64>,
        display_order: u32,
    ) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let notice =
                match create_account_task(&client, name, wallet, networks, display_order).await {
                    Ok(name) => format!("Created account {name}"),
                    Err(error) => format!("Create account failed: {error}"),
                };
            let _ = tx.send(BackgroundUpdate::Notice { generation, notice });
            spawn_layout_fetch(client, tx, generation);
        });
    }

    pub fn create_account_from_key(
        &self,
        generation: u64,
        name: String,
        key: String,
        networks: Vec<u64>,
        display_order: u32,
    ) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let derived = client
                .call_typed::<DerivationFromPrivateKeyMethod>(DerivePrivateKeyParams { input: key })
                .await;
            let notice = match derived {
                Ok(address) => {
                    match create_account_task(
                        &client,
                        name,
                        NewAccountWallet::Eoa(address),
                        networks,
                        display_order,
                    )
                    .await
                    {
                        Ok(name) => format!("Created account {name}"),
                        Err(error) => format!("Create account failed: {error}"),
                    }
                }
                Err(error) => format!("Key derivation failed: {error:#}"),
            };
            let _ = tx.send(BackgroundUpdate::Notice { generation, notice });
            spawn_layout_fetch(client, tx, generation);
        });
    }

    pub fn rename_account(&self, generation: u64, account_id: u64, name: String) {
        self.update_account(
            generation,
            account_id,
            koi::models::account::AccountUpdate {
                name: Some(name),
                networks: None,
                metadata: None,
            },
        );
    }

    pub fn update_account_networks(&self, generation: u64, account_id: u64, networks: Vec<u64>) {
        self.update_account(
            generation,
            account_id,
            koi::models::account::AccountUpdate {
                name: None,
                networks: Some(networks.into_iter().map(NetworkIdentity).collect()),
                metadata: None,
            },
        );
    }

    fn update_account(
        &self,
        generation: u64,
        account_id: u64,
        input: koi::models::account::AccountUpdate,
    ) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let notice = match client
                .call_typed::<AccountUpdateMethod>(AccountUpdateParams {
                    account_identity: AccountIdentity(account_id),
                    input,
                })
                .await
            {
                Ok(account) => format!("Updated account {}", account.name),
                Err(error) => format!("Update account failed: {error:#}"),
            };
            let _ = tx.send(BackgroundUpdate::Notice { generation, notice });
            spawn_layout_fetch(client, tx, generation);
        });
    }

    pub fn delete_account(&self, generation: u64, account_id: u64) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let notice = match client
                .call_typed::<AccountDeleteMethod>(AccountParams {
                    account_identity: AccountIdentity(account_id),
                })
                .await
            {
                Ok(()) => "Deleted account".to_string(),
                Err(error) => format!("Delete account failed: {error:#}"),
            };
            let _ = tx.send(BackgroundUpdate::Notice { generation, notice });
            spawn_layout_fetch(client, tx, generation);
        });
    }

    pub fn fetch_account_assets(&self, generation: u64, account_id: u64, unlink: bool) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            match client
                .call_typed::<AccountAssetListMethod>(AccountParams {
                    account_identity: AccountIdentity(account_id),
                })
                .await
            {
                Ok(identities) => {
                    let _ = tx.send(BackgroundUpdate::AccountAssets {
                        generation,
                        account_id,
                        unlink,
                        identities: identities
                            .into_iter()
                            .map(|identity| identity.to_string())
                            .collect(),
                    });
                }
                Err(error) => {
                    let _ = tx.send(BackgroundUpdate::Notice {
                        generation,
                        notice: format!("Account assets: {error:#}"),
                    });
                }
            }
        });
    }

    pub fn account_asset_action(
        &self,
        generation: u64,
        account_id: u64,
        identity: String,
        unlink: bool,
        display_currency: String,
    ) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let params = identity.parse().map(|asset_identity| AccountAssetParams {
                account_identity: AccountIdentity(account_id),
                asset_identity,
            });
            let notice = match params {
                Ok(params) if unlink => {
                    match client.call_typed::<AccountAssetRemoveMethod>(params).await {
                        Ok(()) => format!("Unlinked {identity}"),
                        Err(error) => format!("Unlink failed: {error:#}"),
                    }
                }
                Ok(params) => match client.call_typed::<AccountAssetAddMethod>(params).await {
                    Ok(()) => format!("Linked {identity}"),
                    Err(error) => format!("Link failed: {error:#}"),
                },
                Err(error) => format!("Invalid asset identity: {error}"),
            };
            let _ = tx.send(BackgroundUpdate::Notice { generation, notice });
            spawn_balance_fetch(client, tx, generation, account_id, display_currency, true);
        });
    }

    pub fn update_endpoint(
        &self,
        generation: u64,
        network_id: u64,
        endpoint_id: i32,
        update: koi::models::network::endpoint::NetworkEndpointUpdate,
        network_ids: Vec<u64>,
    ) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let notice = match client
                .call_typed::<koi::models::network::rpc::EndpointUpdate>(
                    koi::models::network::rpc::EndpointUpdateParams {
                        network_identity: koi::models::network::identity::NetworkIdentity(
                            network_id,
                        ),
                        endpoint_identity: endpoint_id,
                        input: update,
                    },
                )
                .await
            {
                Ok(endpoint) => format!("Updated endpoint #{}", endpoint.endpoint_identity),
                Err(error) => format!("Update endpoint failed: {error:#}"),
            };
            let _ = tx.send(BackgroundUpdate::Notice { generation, notice });
            spawn_settings_fetch(client, tx, generation, network_ids);
        });
    }

    pub fn update_network(
        &self,
        generation: u64,
        network_id: u64,
        update: koi::models::network::NetworkUpdate,
        network_ids: Vec<u64>,
    ) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let notice = match client
                .call_typed::<koi::models::network::rpc::NetworkUpdate>(
                    koi::models::network::rpc::NetworkUpdateParams {
                        network_identity: NetworkIdentity(network_id),
                        input: update,
                    },
                )
                .await
            {
                Ok(network) => format!("Updated network {}", network.network_name),
                Err(error) => format!("Update network failed: {error:#}"),
            };
            let _ = tx.send(BackgroundUpdate::Notice { generation, notice });
            if let Ok(networks) = client.networks().await {
                let _ = tx.send(BackgroundUpdate::NetworksLoaded {
                    generation,
                    networks,
                    notice: None,
                });
            }
            spawn_settings_fetch(client, tx, generation, network_ids);
        });
    }

    pub fn delete_network(&self, generation: u64, network_id: u64, network_ids: Vec<u64>) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let notice = match client
                .call_typed::<koi::models::network::rpc::NetworkDelete>(
                    koi::models::network::rpc::NetworkParams {
                        network_identity: NetworkIdentity(network_id),
                    },
                )
                .await
            {
                Ok(()) => format!("Deleted network {network_id}"),
                Err(error) => format!("Delete network failed: {error:#}"),
            };
            let _ = tx.send(BackgroundUpdate::Notice { generation, notice });
            if let Ok(networks) = client.networks().await {
                let _ = tx.send(BackgroundUpdate::NetworksLoaded {
                    generation,
                    networks,
                    notice: None,
                });
            }
            spawn_settings_fetch(client, tx, generation, network_ids);
        });
    }

    pub fn update_asset(
        &self,
        generation: u64,
        identity: String,
        update: koi::models::asset::AssetUpdate,
        network_ids: Vec<u64>,
    ) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let notice = match identity.parse() {
                Ok(asset_identity) => {
                    match client
                        .call_typed::<koi::models::asset::rpc::AssetUpdate>(
                            koi::models::asset::rpc::AssetUpdateParams {
                                asset_identity,
                                input: update,
                            },
                        )
                        .await
                    {
                        Ok(asset) => format!("Updated asset {}", asset.asset_identity),
                        Err(error) => format!("Update asset failed: {error:#}"),
                    }
                }
                Err(error) => format!("Invalid asset identity: {error}"),
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

    pub fn discover_quoter(&self, generation: u64, token_a: String, token_b: Option<String>) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let parsed = token_a.parse().and_then(|a| {
                token_b
                    .as_deref()
                    .map(str::parse)
                    .transpose()
                    .map(|b| (a, b))
            });
            let result = match parsed {
                Ok((parsed_a, parsed_b)) => client
                    .call_typed::<koi::models::quoter::rpc::QuoterDiscover>(
                        koi::models::quoter::rpc::QuoterDiscoverParams {
                            input: koi::models::quoter::discover::QuoterDiscovery {
                                token_a: parsed_a,
                                token_b: parsed_b,
                            },
                        },
                    )
                    .await
                    .map_err(|error| format!("{error:#}")),
                Err(error) => Err(format!("invalid asset identity: {error}")),
            };
            let _ = tx.send(BackgroundUpdate::QuoterDiscovered {
                generation,
                token_a,
                token_b,
                result,
            });
        });
    }

    pub fn create_quoter(
        &self,
        generation: u64,
        token_a: String,
        token_b: String,
        config: koi::models::quoter::QuoterConfig,
        network_ids: Vec<u64>,
    ) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let parsed = token_a
                .parse()
                .and_then(|a| token_b.parse().map(|b| (a, b)));
            let notice = match parsed {
                Ok((parsed_a, parsed_b)) => {
                    let name = format!("{token_a} → {token_b}");
                    match client
                        .call_typed::<koi::models::quoter::rpc::QuoterCreate>(
                            koi::models::quoter::rpc::QuoterCreateParams {
                                input: koi::models::quoter::QuoterCreate {
                                    quoter_name: name,
                                    token_a: parsed_a,
                                    token_b: parsed_b,
                                    config,
                                    enabled: true,
                                    watch: false,
                                },
                            },
                        )
                        .await
                    {
                        Ok(quoter) => format!("Created quoter {}", quoter.quoter_name),
                        Err(error) => format!("Create quoter failed: {error:#}"),
                    }
                }
                Err(error) => format!("Invalid asset identity: {error}"),
            };
            let _ = tx.send(BackgroundUpdate::Notice { generation, notice });
            spawn_settings_fetch(client, tx, generation, network_ids);
        });
    }

    pub fn toggle_quoter(
        &self,
        generation: u64,
        quoter_identity: String,
        enabled: bool,
        network_ids: Vec<u64>,
    ) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let notice = match client
                .call_typed::<koi::models::quoter::rpc::QuoterUpdate>(
                    koi::models::quoter::rpc::QuoterUpdateParams {
                        quoter_identity,
                        input: koi::models::quoter::QuoterUpdate {
                            quoter_name: None,
                            token_a: None,
                            token_b: None,
                            config: None,
                            enabled: Some(enabled),
                            watch: None,
                        },
                    },
                )
                .await
            {
                Ok(quoter) => format!(
                    "{} quoter {}",
                    if enabled { "Enabled" } else { "Disabled" },
                    quoter.quoter_name
                ),
                Err(error) => format!("Quoter update failed: {error:#}"),
            };
            let _ = tx.send(BackgroundUpdate::Notice { generation, notice });
            spawn_settings_fetch(client, tx, generation, network_ids);
        });
    }

    pub fn fetch_network_presets(&self, generation: u64) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            match client.network_presets().await {
                Ok(presets) => {
                    let _ = tx.send(BackgroundUpdate::NetworkPresets {
                        generation,
                        presets,
                    });
                }
                Err(error) => {
                    let _ = tx.send(BackgroundUpdate::Notice {
                        generation,
                        notice: format!("Network presets: {error:#}"),
                    });
                }
            }
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
                next_id,
            });
        });
    }

    pub fn create_asset(
        &self,
        generation: u64,
        asset: koi::models::asset::Asset,
        network_ids: Vec<u64>,
    ) {
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

    pub fn create_network(
        &self,
        generation: u64,
        network: koi::models::network::Network,
        network_ids: Vec<u64>,
    ) {
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

fn spawn_asset_icon_fetch(
    client: ApiClient,
    tx: mpsc::UnboundedSender<BackgroundUpdate>,
    generation: u64,
    asset_identity: koi::models::asset::identity::AssetIdentity,
) {
    tokio::spawn(async move {
        let identity = asset_identity.to_string();
        let icon = client
            .call_typed::<AssetIconMethod>(koi::models::asset::rpc::AssetParams { asset_identity })
            .await
            .ok()
            .flatten();
        let _ = tx.send(BackgroundUpdate::AssetIcon {
            generation,
            identity,
            icon,
        });
    });
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
    display_currency: String,
    fresh: bool,
) {
    tokio::spawn(async move {
        let state = match client
            .account_balances(account_id, &display_currency, fresh)
            .await
        {
            Ok(balances) => ResourceState::Ready(balances),
            Err(error) => ResourceState::Error(error.to_string()),
        };
        let _ = tx.send(BackgroundUpdate::Balance {
            generation,
            account_id,
            display_currency,
            state,
            refreshing: false,
        });
    });
}

fn spawn_balance_swap_fetch(
    client: ApiClient,
    tx: mpsc::UnboundedSender<BackgroundUpdate>,
    generation: u64,
    account_id: u64,
    display_currency: String,
) {
    tokio::spawn(async move {
        let quick = client
            .account_balances(account_id, &display_currency, false)
            .await
            .ok();
        if let Some(balances) = &quick {
            let _ = tx.send(BackgroundUpdate::Balance {
                generation,
                account_id,
                display_currency: display_currency.clone(),
                state: ResourceState::Ready(balances.clone()),
                refreshing: true,
            });
        }

        let state = match client
            .account_balances(account_id, &display_currency, true)
            .await
        {
            Ok(balances) => ResourceState::Ready(balances),
            Err(error) => match quick {
                // keep the quick values on screen instead of replacing them with an error
                Some(balances) => {
                    let _ = tx.send(BackgroundUpdate::Notice {
                        generation,
                        notice: format!("Balance refresh: {error:#}"),
                    });
                    ResourceState::Ready(balances)
                }
                None => ResourceState::Error(error.to_string()),
            },
        };
        let _ = tx.send(BackgroundUpdate::Balance {
            generation,
            account_id,
            display_currency,
            state,
            refreshing: false,
        });
    });
}

fn spawn_layout_fetch(
    client: ApiClient,
    tx: mpsc::UnboundedSender<BackgroundUpdate>,
    generation: u64,
) {
    tokio::spawn(async move {
        match client.account_layout().await {
            Ok(layout) => {
                let _ = tx.send(BackgroundUpdate::LayoutLoaded {
                    generation,
                    groups: layout.groups,
                    accounts: layout.accounts,
                    notice: None,
                });
            }
            Err(error) => {
                let _ = tx.send(BackgroundUpdate::Notice {
                    generation,
                    notice: format!("Accounts: {error:#}"),
                });
            }
        }
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
            Ok(vendors) => vendors
                .into_iter()
                .map(|vendor| vendor.to_string())
                .collect(),
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

async fn create_account_task(
    client: &ApiClient,
    name: String,
    wallet: NewAccountWallet,
    networks: Vec<u64>,
    display_order: u32,
) -> Result<String, String> {
    let parse = |address: &str| {
        address
            .trim()
            .parse()
            .map_err(|error| format!("invalid address: {error}"))
    };
    let metadata = match &wallet {
        NewAccountWallet::View(address) => WalletType::View(ViewWallet {
            evm_address: parse(address)?,
        }),
        NewAccountWallet::Safe(address) => WalletType::Safe(SafeWallet {
            evm_address: parse(address)?,
        }),
        NewAccountWallet::Eoa(address) => WalletType::EOA(EOAWallet {
            evm_address: parse(address)?,
        }),
    };

    let identity = client
        .call_typed::<AccountNextIdentityMethod>(EmptyParams::default())
        .await
        .map_err(|error| format!("{error:#}"))?;
    let account = Account {
        account_identity: identity,
        name,
        networks: networks.into_iter().map(NetworkIdentity).collect(),
        metadata,
        group_id: None,
        display_order,
    };
    client
        .call_typed::<AccountCreateMethod>(AccountCreateParams { input: account })
        .await
        .map(|created| created.name)
        .map_err(|error| format!("{error:#}"))
}

pub fn prepare_refresh_all(app: &mut App) -> u64 {
    app.begin_refresh()
}
