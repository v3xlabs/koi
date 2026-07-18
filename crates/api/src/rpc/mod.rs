use alloy::primitives::U256;
use futures::{StreamExt, future::join_all, stream};
use koi::{
    error::KoiError,
    models::{
        account::{
            Account, AccountUpdate,
            balances::{AccountBalance, AccountBalances},
            derive::{
                default_derivation_path, derive_address_from_private_key,
                derive_addresses_from_mnemonic, generate_mnemonic,
            },
            group::{AccountGroup, AccountGroupCreate, AccountGroupUpdate, GroupIdentity},
            identity::AccountIdentity,
            layout::{AccountLayout, AccountLayoutUpdate},
            metadata::WalletType,
        },
        asset::{Asset, AssetUpdate, identity::AssetIdentity, metadata::AssetMetadataDiscovery},
        network::{
            Network, NetworkUpdate,
            endpoint::{NetworkEndpoint, NetworkEndpointUpdate, provider::RpcStatus},
            identity::NetworkIdentity,
            metadata::NetworkMetadataDiscovery,
            pool::RpcPoolStats,
        },
        quoter::{
            Quoter, QuoterCreate, QuoterUpdate,
            discover::{QuoterDiscovery, QuoterDiscoveryResponse},
        },
        tx::{
            Tx, TxBase,
            decode::{
                DecodeTransactionRequest, DecodeTransactionResponse, SimulateTransactionRequest,
                SimulateTransactionResponse, decode_transaction,
            },
            simulate::simulate_transaction,
        },
        vendor::flags::{VendorFlag, VendorFlagInfo},
    },
    state::AppState,
    vendor::safe_wallet::tx::fetch_safewallet_tx,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Map, Value, json};
use ts_rs::TS;

pub const JSON_RPC_VERSION: &str = "2.0";
pub const MAX_MESSAGE_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_BATCH_ENTRIES: usize = 128;
pub const MAX_IN_FLIGHT_CALLS: usize = 64;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum RpcErrorKind {
    InvalidInput,
    NotFound,
    Conflict,
    Unavailable,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
pub struct RpcErrorData {
    pub kind: RpcErrorKind,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
pub struct RpcErrorObject {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<RpcErrorData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, Default)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct EmptyParams {}

pub trait RpcMethod {
    type Params: Serialize + DeserializeOwned;
    type Output: Serialize + DeserializeOwned;
    const NAME: &'static str;
}

macro_rules! method {
    ($marker:ident, $params:ty, $output:ty, $name:literal) => {
        pub struct $marker;

        impl RpcMethod for $marker {
            type Params = $params;
            type Output = $output;
            const NAME: &'static str = $name;
        }
    };
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct AccountParams {
    #[ts(type = "number")]
    pub account_identity: AccountIdentity,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct AssetParams {
    #[ts(type = "string")]
    pub asset_identity: AssetIdentity,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct NetworkParams {
    #[ts(type = "number")]
    pub network_identity: NetworkIdentity,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct AccountAssetParams {
    #[ts(type = "number")]
    pub account_identity: AccountIdentity,
    #[ts(type = "string")]
    pub asset_identity: AssetIdentity,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct AccountAssetBalanceParams {
    #[ts(type = "number")]
    pub account_identity: AccountIdentity,
    #[ts(type = "string")]
    pub asset_identity: AssetIdentity,
    #[ts(type = "string")]
    pub display_currency: AssetIdentity,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct AccountBalancesParams {
    #[ts(type = "number")]
    pub account_identity: AccountIdentity,
    #[ts(type = "string")]
    pub display_currency: AssetIdentity,
    #[serde(default)]
    pub fresh: bool,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct AccountCreateParams {
    #[ts(type = "Account")]
    pub input: Account,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct AccountUpdateParams {
    #[ts(type = "number")]
    pub account_identity: AccountIdentity,
    #[ts(type = "AccountUpdate")]
    pub input: AccountUpdate,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct LayoutUpdateParams {
    #[ts(type = "AccountLayoutUpdate")]
    pub input: AccountLayoutUpdate,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct GroupCreateParams {
    #[ts(type = "AccountGroupCreate")]
    pub input: AccountGroupCreate,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct GroupUpdateParams {
    #[ts(type = "number")]
    pub group_identity: GroupIdentity,
    #[ts(type = "AccountGroupUpdate")]
    pub input: AccountGroupUpdate,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct GroupParams {
    #[ts(type = "number")]
    pub group_identity: GroupIdentity,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct DeriveMnemonicInput {
    pub mnemonic: String,
    pub paths: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct DeriveMnemonicParams {
    pub input: DeriveMnemonicInput,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct DeriveMnemonicResult {
    pub path: String,
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct DerivePrivateKeyParams {
    pub input: String,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct AssetCreateParams {
    #[ts(type = "Asset")]
    pub input: Asset,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct AssetUpdateParams {
    #[ts(type = "string")]
    pub asset_identity: AssetIdentity,
    #[ts(type = "AssetUpdate")]
    pub input: AssetUpdate,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct AssetQuoteParams {
    #[ts(type = "string")]
    pub asset_identity: AssetIdentity,
    #[serde(default)]
    #[ts(optional)]
    #[ts(type = "string | null")]
    pub display_asset: Option<AssetIdentity>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct NetworkCreateParams {
    #[ts(type = "Network")]
    pub input: Network,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct NetworkUpdateParams {
    #[ts(type = "number")]
    pub network_identity: NetworkIdentity,
    #[ts(type = "NetworkUpdate")]
    pub input: NetworkUpdate,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct EndpointParams {
    #[ts(type = "number")]
    pub network_identity: NetworkIdentity,
    pub endpoint_identity: i32,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct EndpointCreateParams {
    #[ts(type = "number")]
    pub network_identity: NetworkIdentity,
    #[ts(type = "NetworkEndpoint")]
    pub input: NetworkEndpoint,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct EndpointUpdateParams {
    #[ts(type = "number")]
    pub network_identity: NetworkIdentity,
    pub endpoint_identity: i32,
    #[ts(type = "NetworkEndpointUpdate")]
    pub input: NetworkEndpointUpdate,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct SimulateParams {
    #[ts(type = "number")]
    pub network_identity: NetworkIdentity,
    #[ts(type = "SimulateTransactionRequest")]
    pub input: SimulateTransactionRequest,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct DecodeParams {
    #[ts(type = "number")]
    pub network_identity: NetworkIdentity,
    #[ts(type = "DecodeTransactionRequest")]
    pub input: DecodeTransactionRequest,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct QuoterParams {
    pub quoter_identity: String,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct QuoterCreateParams {
    #[ts(type = "QuoterCreate")]
    pub input: QuoterCreate,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct QuoterUpdateParams {
    pub quoter_identity: String,
    #[ts(type = "QuoterUpdate")]
    pub input: QuoterUpdate,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct QuoterDiscoverParams {
    #[ts(type = "QuoterDiscovery")]
    pub input: QuoterDiscovery,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct VendorParams {
    #[ts(type = "VendorFlag")]
    pub flag: VendorFlag,
}

pub mod methods {
    use super::*;

    method!(SystemPing, EmptyParams, String, "system.ping");
    method!(AccountList, EmptyParams, Vec<Account>, "account.list");
    method!(AccountGet, AccountParams, Account, "account.get");
    method!(
        AccountCreate,
        AccountCreateParams,
        Account,
        "account.create"
    );
    method!(
        AccountNextIdentity,
        EmptyParams,
        AccountIdentity,
        "account.nextIdentity"
    );
    method!(
        AccountUpdate,
        AccountUpdateParams,
        Account,
        "account.update"
    );
    method!(AccountDelete, AccountParams, (), "account.delete");
    method!(
        AccountAssetList,
        AccountParams,
        Vec<AssetIdentity>,
        "account.asset.list"
    );
    method!(AccountAssetAdd, AccountAssetParams, (), "account.asset.add");
    method!(
        AccountAssetRemove,
        AccountAssetParams,
        (),
        "account.asset.remove"
    );
    method!(
        AccountAssetBalance,
        AccountAssetBalanceParams,
        AccountBalance,
        "account.asset.balance"
    );
    method!(
        AccountBalanceList,
        AccountBalancesParams,
        AccountBalances,
        "account.balance.list"
    );
    method!(
        AccountLayoutGet,
        EmptyParams,
        AccountLayout,
        "account.layout.get"
    );
    method!(
        AccountLayoutUpdate,
        LayoutUpdateParams,
        AccountLayout,
        "account.layout.update"
    );
    method!(
        AccountGroupCreate,
        GroupCreateParams,
        AccountGroup,
        "account.group.create"
    );
    method!(
        AccountGroupUpdate,
        GroupUpdateParams,
        AccountGroup,
        "account.group.update"
    );
    method!(AccountGroupDelete, GroupParams, (), "account.group.delete");
    method!(
        AccountTransactionList,
        AccountParams,
        Vec<Tx>,
        "account.transaction.list"
    );
    method!(
        AccountTransactionPending,
        AccountParams,
        Vec<Tx>,
        "account.transaction.pending"
    );
    method!(
        AccountMnemonicGenerate,
        EmptyParams,
        String,
        "account.mnemonic.generate"
    );
    method!(
        AccountDerivationDefaultPath,
        EmptyParams,
        String,
        "account.derivation.defaultPath"
    );
    method!(
        AccountDerivationFromMnemonic,
        DeriveMnemonicParams,
        Vec<DeriveMnemonicResult>,
        "account.derivation.fromMnemonic"
    );
    method!(
        AccountDerivationFromPrivateKey,
        DerivePrivateKeyParams,
        String,
        "account.derivation.fromPrivateKey"
    );
    method!(AssetList, EmptyParams, Vec<Asset>, "asset.list");
    method!(AssetGet, AssetParams, Asset, "asset.get");
    method!(AssetCreate, AssetCreateParams, Asset, "asset.create");
    method!(AssetUpdate, AssetUpdateParams, Asset, "asset.update");
    method!(AssetDelete, AssetParams, (), "asset.delete");
    method!(
        AssetDiscoverMetadata,
        AssetParams,
        AssetMetadataDiscovery,
        "asset.discoverMetadata"
    );
    method!(AssetQuote, AssetQuoteParams, String, "asset.quote");
    method!(NetworkList, EmptyParams, Vec<Network>, "network.list");
    method!(NetworkGet, NetworkParams, Network, "network.get");
    method!(
        NetworkCreate,
        NetworkCreateParams,
        Network,
        "network.create"
    );
    method!(
        NetworkUpdate,
        NetworkUpdateParams,
        Network,
        "network.update"
    );
    method!(NetworkDelete, NetworkParams, (), "network.delete");
    method!(
        NetworkListPresets,
        EmptyParams,
        Vec<Network>,
        "network.listPresets"
    );
    method!(
        NetworkDiscoverMetadata,
        NetworkParams,
        NetworkMetadataDiscovery,
        "network.discoverMetadata"
    );
    method!(
        NetworkRpcStats,
        NetworkParams,
        RpcPoolStats,
        "network.rpcStats"
    );
    method!(
        EndpointList,
        NetworkParams,
        Vec<NetworkEndpoint>,
        "network.endpoint.list"
    );
    method!(
        EndpointGet,
        EndpointParams,
        NetworkEndpoint,
        "network.endpoint.get"
    );
    method!(
        EndpointCreate,
        EndpointCreateParams,
        NetworkEndpoint,
        "network.endpoint.create"
    );
    method!(
        EndpointUpdate,
        EndpointUpdateParams,
        NetworkEndpoint,
        "network.endpoint.update"
    );
    method!(
        EndpointDelete,
        EndpointParams,
        (),
        "network.endpoint.delete"
    );
    method!(
        EndpointNextIdentity,
        NetworkParams,
        i32,
        "network.endpoint.nextIdentity"
    );
    method!(
        EndpointStatus,
        EndpointParams,
        RpcStatus,
        "network.endpoint.status"
    );
    method!(
        TransactionSimulate,
        SimulateParams,
        SimulateTransactionResponse,
        "transaction.simulate"
    );
    method!(
        TransactionDecode,
        DecodeParams,
        DecodeTransactionResponse,
        "transaction.decode"
    );
    method!(QuoterList, EmptyParams, Vec<Quoter>, "quoter.list");
    method!(QuoterGet, QuoterParams, Quoter, "quoter.get");
    method!(QuoterCreate, QuoterCreateParams, Quoter, "quoter.create");
    method!(QuoterUpdate, QuoterUpdateParams, Quoter, "quoter.update");
    method!(
        QuoterDiscover,
        QuoterDiscoverParams,
        QuoterDiscoveryResponse,
        "quoter.discover"
    );
    method!(
        VendorListEnabled,
        EmptyParams,
        Vec<VendorFlag>,
        "vendor.listEnabled"
    );
    method!(
        VendorListAll,
        EmptyParams,
        Vec<VendorFlagInfo>,
        "vendor.listAll"
    );
    method!(VendorEnable, VendorParams, (), "vendor.enable");
    method!(VendorDisable, VendorParams, (), "vendor.disable");
}

#[derive(Clone)]
pub struct Dispatcher {
    state: AppState,
}

impl Dispatcher {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub async fn call<M: RpcMethod>(&self, params: M::Params) -> Result<M::Output, RpcErrorObject> {
        let params = serde_json::to_value(params).map_err(internal_error)?;
        let value = self.dispatch_method(M::NAME, params).await?;
        serde_json::from_value(value).map_err(internal_error)
    }

    pub async fn process_message(&self, message: &str) -> Option<String> {
        let value = match serde_json::from_str::<Value>(message) {
            Ok(value) => value,
            Err(_) => return serialize_response(error_response(Value::Null, parse_error())),
        };

        let response = match value {
            Value::Array(values) if values.is_empty() => Some(error_response(
                Value::Null,
                invalid_request("batch must not be empty"),
            )),
            Value::Array(values) if values.len() > MAX_BATCH_ENTRIES => Some(error_response(
                Value::Null,
                invalid_request("batch exceeds 128 entries"),
            )),
            Value::Array(values) => {
                let responses =
                    join_all(values.into_iter().map(|value| self.process_request(value)))
                        .await
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>();
                (!responses.is_empty()).then(|| Value::Array(responses))
            }
            value => self.process_request(value).await,
        };

        response.and_then(serialize_response)
    }

    pub async fn process_request(&self, value: Value) -> Option<Value> {
        let Value::Object(mut object) = value else {
            return Some(error_response(
                Value::Null,
                invalid_request("request must be an object"),
            ));
        };

        let id_present = object.contains_key("id");
        let id = object.remove("id").unwrap_or(Value::Null);
        if id_present && !matches!(id, Value::Null | Value::String(_) | Value::Number(_)) {
            return Some(error_response(
                Value::Null,
                invalid_request("invalid request id"),
            ));
        }

        if object.remove("jsonrpc") != Some(Value::String(JSON_RPC_VERSION.to_string())) {
            return Some(error_response(id, invalid_request("jsonrpc must be 2.0")));
        }

        let Some(Value::String(method)) = object.remove("method") else {
            return Some(error_response(
                id,
                invalid_request("method must be a string"),
            ));
        };
        let params = object
            .remove("params")
            .unwrap_or_else(|| Value::Object(Map::new()));
        if !object.is_empty() {
            return Some(error_response(
                id,
                invalid_request("unknown request fields"),
            ));
        }

        let result = self.dispatch_method(&method, params).await;
        if !id_present {
            return None;
        }

        Some(match result {
            Ok(result) => json!({ "jsonrpc": JSON_RPC_VERSION, "id": id, "result": result }),
            Err(error) => error_response(id, error),
        })
    }

    async fn dispatch_method(&self, method: &str, params: Value) -> Result<Value, RpcErrorObject> {
        macro_rules! run {
            ($params:ty, |_| $body:expr) => {{
                let _: $params = parse_params(params)?;
                let result: Result<_, KoiError> = $body.await;
                serialize_domain_result(result)
            }};
            ($params:ty, |$binding:ident| $body:expr) => {{
                let $binding: $params = parse_params(params)?;
                let result: Result<_, KoiError> = $body.await;
                serialize_domain_result(result)
            }};
        }

        match method {
            "system.ping" => run!(EmptyParams, |_| async { Ok("OK".to_string()) }),
            "account.list" => run!(EmptyParams, |_| async {
                Account::all(&self.state.database).await
            }),
            "account.get" => run!(AccountParams, |p| async {
                Account::get_by_id(&self.state.database, p.account_identity).await
            }),
            "account.create" => run!(AccountCreateParams, |p| async {
                Account::create(&self.state.database, p.input).await
            }),
            "account.nextIdentity" => run!(EmptyParams, |_| async {
                Account::get_next_identity(&self.state.database).await
            }),
            "account.update" => run!(AccountUpdateParams, |p| async {
                Account::update(&self.state.database, p.account_identity, p.input).await
            }),
            "account.delete" => run!(AccountParams, |p| async {
                Account::delete(&self.state.database, p.account_identity).await
            }),
            "account.asset.list" => run!(AccountParams, |p| async {
                Account::get_assets(&self.state.database, p.account_identity).await
            }),
            "account.asset.add" => run!(AccountAssetParams, |p| async {
                Account::add_asset(&self.state.database, p.account_identity, p.asset_identity).await
            }),
            "account.asset.remove" => run!(AccountAssetParams, |p| async {
                Account::remove_asset(&self.state.database, p.account_identity, p.asset_identity)
                    .await
            }),
            "account.asset.balance" => run!(AccountAssetBalanceParams, |p| async {
                let account = Account::get_by_id(&self.state.database, p.account_identity).await?;
                account
                    .fetch_asset_balance(&self.state, &p.asset_identity, &p.display_currency)
                    .await
            }),
            "account.balance.list" => run!(AccountBalancesParams, |p| async {
                let account = Account::get_by_id(&self.state.database, p.account_identity).await?;
                self.state
                    .balances
                    .get_balances(&self.state, &account, &p.display_currency, p.fresh)
                    .await
            }),
            "account.layout.get" => run!(EmptyParams, |_| async {
                AccountLayout::get(&self.state.database).await
            }),
            "account.layout.update" => run!(LayoutUpdateParams, |p| async {
                AccountLayout::update(&self.state.database, p.input).await
            }),
            "account.group.create" => run!(GroupCreateParams, |p| async {
                AccountGroup::create(&self.state.database, p.input.name).await
            }),
            "account.group.update" => run!(GroupUpdateParams, |p| async {
                AccountGroup::update(&self.state.database, p.group_identity, p.input).await
            }),
            "account.group.delete" => run!(GroupParams, |p| async {
                AccountGroup::delete(&self.state.database, p.group_identity).await
            }),
            "account.transaction.list" => run!(AccountParams, |p| async {
                account_transactions(&self.state, p.account_identity).await
            }),
            "account.transaction.pending" => run!(AccountParams, |p| async {
                let account = Account::get_by_id(&self.state.database, p.account_identity).await?;
                account
                    .metadata
                    .unwrap_address()
                    .ok_or_else(|| KoiError::InvalidInput("account has no address".to_string()))?;
                Ok(Vec::<Tx>::new())
            }),
            "account.mnemonic.generate" => run!(EmptyParams, |_| async { generate_mnemonic() }),
            "account.derivation.defaultPath" => run!(EmptyParams, |_| async {
                Ok(default_derivation_path().to_string())
            }),
            "account.derivation.fromMnemonic" => run!(DeriveMnemonicParams, |p| async {
                derive_addresses_from_mnemonic(&p.input.mnemonic, &p.input.paths).map(|values| {
                    values
                        .into_iter()
                        .map(|(path, address)| DeriveMnemonicResult {
                            path,
                            address: address.to_checksum(None),
                        })
                        .collect::<Vec<_>>()
                })
            }),
            "account.derivation.fromPrivateKey" => run!(DerivePrivateKeyParams, |p| async {
                derive_address_from_private_key(&p.input).map(|address| address.to_checksum(None))
            }),
            "asset.list" => run!(EmptyParams, |_| async {
                Asset::all(&self.state.database).await
            }),
            "asset.get" => run!(AssetParams, |p| async {
                Asset::get_by_id(&self.state.database, &p.asset_identity).await
            }),
            "asset.create" => run!(AssetCreateParams, |p| async {
                Asset::create(&self.state.database, p.input).await
            }),
            "asset.update" => run!(AssetUpdateParams, |p| async {
                Asset::update(&self.state.database, &p.asset_identity, p.input).await
            }),
            "asset.delete" => run!(AssetParams, |p| async {
                Asset::delete(&self.state.database, &p.asset_identity).await
            }),
            "asset.discoverMetadata" => run!(AssetParams, |p| async {
                Asset::fetch_metadata(&self.state, &p.asset_identity).await
            }),
            "asset.quote" => run!(AssetQuoteParams, |p| async {
                let network = p
                    .asset_identity
                    .unwrap_network()
                    .ok_or_else(|| KoiError::InvalidInput("asset has no network".to_string()))?;
                let output = p
                    .display_asset
                    .unwrap_or_else(|| AssetIdentity::Fiat("usd".to_string()));
                let asset = Asset::get_by_id(&self.state.database, &p.asset_identity).await?;
                let amount = U256::from(10).pow(U256::from(asset.asset_decimals));
                self.state
                    .quoters
                    .quote(&self.state, &network, &p.asset_identity, &output, amount)
                    .await
                    .map(|quote| quote.to_string())
            }),
            "network.list" => run!(EmptyParams, |_| async {
                Network::all(&self.state.database).await
            }),
            "network.get" => run!(NetworkParams, |p| async {
                Network::get_by_id(&self.state.database, &p.network_identity).await
            }),
            "network.create" => run!(NetworkCreateParams, |p| async {
                Network::create(&self.state.database, p.input).await
            }),
            "network.update" => run!(NetworkUpdateParams, |p| async {
                Network::update(&self.state.database, &p.network_identity, p.input).await
            }),
            "network.delete" => run!(NetworkParams, |p| async {
                Network::delete(&self.state.database, &p.network_identity).await
            }),
            "network.listPresets" => run!(EmptyParams, |_| async { Ok(Network::presets()) }),
            "network.discoverMetadata" => run!(NetworkParams, |p| async {
                Network::fetch_metadata(&self.state, &p.network_identity).await
            }),
            "network.rpcStats" => run!(NetworkParams, |p| async {
                Ok(self.state.networks.get_pool(&p.network_identity).snapshot())
            }),
            "network.endpoint.list" => run!(NetworkParams, |p| async {
                NetworkEndpoint::get_by_network_id(&self.state.database, &p.network_identity).await
            }),
            "network.endpoint.get" => run!(EndpointParams, |p| async {
                NetworkEndpoint::get_by_id(
                    &self.state.database,
                    &p.network_identity,
                    &p.endpoint_identity,
                )
                .await
            }),
            "network.endpoint.create" => run!(EndpointCreateParams, |p| async {
                if p.input.network_identity != p.network_identity {
                    return Err(KoiError::InvalidInput(
                        "endpoint network does not match parameters".to_string(),
                    ));
                }
                NetworkEndpoint::create(&self.state.database, p.input).await
            }),
            "network.endpoint.update" => run!(EndpointUpdateParams, |p| async {
                let endpoint = NetworkEndpoint::update(
                    &self.state.database,
                    &p.network_identity,
                    &p.endpoint_identity,
                    p.input,
                )
                .await?;
                self.state
                    .networks
                    .get_pool(&p.network_identity)
                    .get_rpc(&p.endpoint_identity, &self.state)
                    .await
                    .update(&endpoint)
                    .await
                    .map_err(KoiError::from)?;
                Ok(endpoint)
            }),
            "network.endpoint.delete" => run!(EndpointParams, |p| async {
                NetworkEndpoint::delete(
                    &self.state.database,
                    &p.network_identity,
                    &p.endpoint_identity,
                )
                .await?;
                self.state
                    .networks
                    .get_pool(&p.network_identity)
                    .remove_endpoint(&p.endpoint_identity);
                Ok(())
            }),
            "network.endpoint.nextIdentity" => run!(NetworkParams, |_| async {
                NetworkEndpoint::get_next_id(&self.state.database).await
            }),
            "network.endpoint.status" => run!(EndpointParams, |p| async {
                Ok(self
                    .state
                    .networks
                    .get_pool(&p.network_identity)
                    .get_rpc(&p.endpoint_identity, &self.state)
                    .await
                    .get_status()
                    .await)
            }),
            "transaction.simulate" => run!(SimulateParams, |p| async {
                Network::get_by_id(&self.state.database, &p.network_identity).await?;
                simulate_transaction(&self.state, &p.network_identity, &p.input).await
            }),
            "transaction.decode" => run!(DecodeParams, |p| async {
                Network::get_by_id(&self.state.database, &p.network_identity).await?;
                decode_transaction(&self.state, &p.network_identity, &p.input).await
            }),
            "quoter.list" => run!(EmptyParams, |_| async {
                Quoter::all(&self.state.database).await
            }),
            "quoter.get" => run!(QuoterParams, |p| async {
                Quoter::get_by_id(&self.state.database, &p.quoter_identity).await
            }),
            "quoter.create" => run!(QuoterCreateParams, |p| async {
                let quoter = Quoter::insert(&self.state.database, p.input).await?;
                self.state.quoters.build_graph(&self.state.database).await?;
                Ok(quoter)
            }),
            "quoter.update" => run!(QuoterUpdateParams, |p| async {
                let quoter =
                    Quoter::update(&self.state.database, &p.quoter_identity, p.input).await?;
                self.state.quoters.build_graph(&self.state.database).await?;
                Ok(quoter)
            }),
            "quoter.discover" => run!(QuoterDiscoverParams, |p| async {
                p.input.discover(&self.state).await
            }),
            "vendor.listEnabled" => run!(EmptyParams, |_| async { Ok(self.state.vendors.all()) }),
            "vendor.listAll" => run!(EmptyParams, |_| async { Ok(VendorFlag::all()) }),
            "vendor.enable" => run!(VendorParams, |p| async {
                self.state
                    .vendors
                    .set_flag(&p.flag, true, &self.state.database)
                    .await
            }),
            "vendor.disable" => run!(VendorParams, |p| async {
                self.state
                    .vendors
                    .set_flag(&p.flag, false, &self.state.database)
                    .await
            }),
            _ => Err(RpcErrorObject {
                code: -32601,
                message: "Method not found".to_string(),
                data: None,
            }),
        }
    }
}

async fn account_transactions(
    state: &AppState,
    identity: AccountIdentity,
) -> Result<Vec<Tx>, KoiError> {
    let account = Account::get_by_id(&state.database, identity).await?;
    let bases = match account.metadata {
        WalletType::Safe(safe) => stream::iter(account.networks)
            .map(|network| async move {
                fetch_safewallet_tx(network, safe.evm_address.0)
                    .await
                    .map(|response| {
                        response
                            .results
                            .into_iter()
                            .filter_map(|tx| tx.try_into().ok())
                            .collect::<Vec<TxBase>>()
                    })
                    .unwrap_or_default()
            })
            .buffered(8)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .flatten()
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    };

    Ok(stream::iter(bases)
        .map(|tx| async move { tx.decode(state).await.ok() })
        .buffered(8)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .flatten()
        .collect())
}

fn parse_params<T: DeserializeOwned>(params: Value) -> Result<T, RpcErrorObject> {
    if !params.is_object() {
        return Err(invalid_params());
    }

    serde_json::from_value(params).map_err(|_| invalid_params())
}

fn invalid_params() -> RpcErrorObject {
    RpcErrorObject {
        code: -32602,
        message: "Invalid params".to_string(),
        data: Some(RpcErrorData {
            kind: RpcErrorKind::InvalidInput,
            message: "parameters do not match the method contract".to_string(),
        }),
    }
}

fn serialize_domain_result<T: Serialize>(
    result: Result<T, KoiError>,
) -> Result<Value, RpcErrorObject> {
    result
        .map_err(application_error)
        .and_then(|value| serde_json::to_value(value).map_err(internal_error))
}

fn application_error(error: KoiError) -> RpcErrorObject {
    let kind = match &error {
        KoiError::InvalidInput(_) | KoiError::Parse(_) | KoiError::AlloyHex(_) => {
            RpcErrorKind::InvalidInput
        }
        KoiError::NotFound(_) => RpcErrorKind::NotFound,
        KoiError::Database(sqlx::Error::RowNotFound) => RpcErrorKind::NotFound,
        KoiError::Conflict(_) => RpcErrorKind::Conflict,
        KoiError::Unavailable(_) | KoiError::Rpc(_) => RpcErrorKind::Unavailable,
        _ => RpcErrorKind::Internal,
    };
    RpcErrorObject {
        code: -32000,
        message: "Application error".to_string(),
        data: Some(RpcErrorData {
            kind,
            message: error.safe_message(),
        }),
    }
}

fn internal_error(error: impl std::fmt::Display) -> RpcErrorObject {
    let _ = error;
    RpcErrorObject {
        code: -32603,
        message: "Internal error".to_string(),
        data: Some(RpcErrorData {
            kind: RpcErrorKind::Internal,
            message: "the response could not be encoded".to_string(),
        }),
    }
}

fn parse_error() -> RpcErrorObject {
    RpcErrorObject {
        code: -32700,
        message: "Parse error".to_string(),
        data: None,
    }
}

fn invalid_request(message: &str) -> RpcErrorObject {
    RpcErrorObject {
        code: -32600,
        message: "Invalid Request".to_string(),
        data: Some(RpcErrorData {
            kind: RpcErrorKind::InvalidInput,
            message: message.to_string(),
        }),
    }
}

fn error_response(id: Value, error: RpcErrorObject) -> Value {
    json!({ "jsonrpc": JSON_RPC_VERSION, "id": id, "error": error })
}

fn serialize_response(value: Value) -> Option<String> {
    serde_json::to_string(&value).ok()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use koi::{
        config::Configuration,
        db::connect,
        models::{
            abi::AbiManager, account::balance_cache::BalanceCacheManager,
            network::manager::NetworkManager, quoter::man::QuoterManager,
            vendor::man::VendorManager,
        },
        state::State,
    };
    use serde_json::Value;
    use tempfile::TempDir;

    use super::*;

    async fn test_dispatcher() -> (TempDir, Dispatcher) {
        let directory = tempfile::tempdir().unwrap();
        let database_path = directory.path().join("rpc.db");
        let database_url = format!("sqlite://{}", database_path.display());
        let database = connect(&database_url, None).await.unwrap();
        let vendors = VendorManager::init(&database).await.unwrap();
        let quoters = QuoterManager::init(&database).await.unwrap();
        let state = Arc::new(State {
            config: Configuration {
                database_url,
                abi_cache_dir: directory.path().join("abis").display().to_string(),
                ..Configuration::default()
            },
            database,
            networks: NetworkManager::default(),
            quoters,
            balances: BalanceCacheManager::new(),
            vendors,
            abis: AbiManager::new(directory.path().join("abis")),
        });

        (directory, Dispatcher::new(state))
    }

    async fn response(dispatcher: &Dispatcher, request: &str) -> Value {
        serde_json::from_str(&dispatcher.process_message(request).await.unwrap()).unwrap()
    }

    #[test]
    fn protocol_constants_match_limits() {
        assert_eq!(MAX_MESSAGE_BYTES, 8_388_608);
        assert_eq!(MAX_BATCH_ENTRIES, 128);
        assert_eq!(MAX_IN_FLIGHT_CALLS, 64);
    }

    #[tokio::test]
    async fn single_request_and_notification_follow_json_rpc() {
        let (_directory, dispatcher) = test_dispatcher().await;
        let single = response(
            &dispatcher,
            r#"{"jsonrpc":"2.0","id":7,"method":"system.ping","params":{}}"#,
        )
        .await;

        assert_eq!(single, json!({"jsonrpc": "2.0", "id": 7, "result": "OK"}));
        assert_eq!(
            dispatcher
                .process_message(r#"{"jsonrpc":"2.0","method":"system.ping","params":{}}"#)
                .await,
            None
        );
    }

    #[tokio::test]
    async fn typed_in_process_dispatch_uses_the_same_method_markers() {
        let (_directory, dispatcher) = test_dispatcher().await;

        assert_eq!(
            dispatcher
                .call::<methods::SystemPing>(EmptyParams::default())
                .await
                .unwrap(),
            "OK"
        );
        assert_eq!(
            dispatcher
                .call::<methods::EndpointNextIdentity>(NetworkParams {
                    network_identity: NetworkIdentity(1),
                })
                .await
                .unwrap(),
            1
        );
    }

    #[tokio::test]
    async fn mixed_batches_are_concurrent_but_response_order_is_stable() {
        let (_directory, dispatcher) = test_dispatcher().await;
        let batch = response(
            &dispatcher,
            r#"[
                {"jsonrpc":"2.0","id":1,"method":"system.ping","params":{}},
                {"jsonrpc":"2.0","method":"system.ping","params":{}},
                {"jsonrpc":"2.0","id":2,"method":"missing","params":{}},
                {"jsonrpc":"2.0","id":3,"method":"system.ping","params":{"extra":true}}
            ]"#,
        )
        .await;

        let entries = batch.as_array().unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0]["id"], 1);
        assert_eq!(entries[0]["result"], "OK");
        assert_eq!(entries[1]["id"], 2);
        assert_eq!(entries[1]["error"]["code"], -32601);
        assert_eq!(entries[2]["id"], 3);
        assert_eq!(entries[2]["error"]["code"], -32602);
    }

    #[tokio::test]
    async fn notification_only_batches_have_no_response() {
        let (_directory, dispatcher) = test_dispatcher().await;

        assert_eq!(
            dispatcher
                .process_message(
                    r#"[
                        {"jsonrpc":"2.0","method":"system.ping","params":{}},
                        {"jsonrpc":"2.0","method":"system.ping","params":{}}
                    ]"#,
                )
                .await,
            None
        );
    }

    #[tokio::test]
    async fn protocol_errors_use_standard_codes() {
        let (_directory, dispatcher) = test_dispatcher().await;

        assert_eq!(response(&dispatcher, "{").await["error"]["code"], -32700);
        assert_eq!(response(&dispatcher, "[]").await["error"]["code"], -32600);
        assert_eq!(
            response(
                &dispatcher,
                r#"{"jsonrpc":"2.0","id":1,"method":"system.ping","params":[]}"#,
            )
            .await["error"]["code"],
            -32602
        );

        let oversized_batch = Value::Array(vec![
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "system.ping",
                "params": {}
            });
            MAX_BATCH_ENTRIES + 1
        ]);
        assert_eq!(
            response(&dispatcher, &oversized_batch.to_string()).await["error"]["code"],
            -32600
        );
    }

    #[tokio::test]
    async fn duplicate_ids_remain_correlated_in_request_order() {
        let (_directory, dispatcher) = test_dispatcher().await;
        let batch = response(
            &dispatcher,
            r#"[
                {"jsonrpc":"2.0","id":9,"method":"system.ping","params":{}},
                {"jsonrpc":"2.0","id":9,"method":"account.list","params":{}}
            ]"#,
        )
        .await;
        let entries = batch.as_array().unwrap();

        assert_eq!(
            entries[0],
            json!({"jsonrpc": "2.0", "id": 9, "result": "OK"})
        );
        assert_eq!(entries[1], json!({"jsonrpc": "2.0", "id": 9, "result": []}));
    }

    #[tokio::test]
    async fn application_errors_have_typed_safe_data() {
        let (_directory, dispatcher) = test_dispatcher().await;
        let missing = response(
            &dispatcher,
            r#"{"jsonrpc":"2.0","id":4,"method":"account.get","params":{"account_identity":999}}"#,
        )
        .await;

        assert_eq!(missing["error"]["code"], -32000);
        assert_eq!(missing["error"]["data"]["kind"], "not_found");
        assert_eq!(missing["error"]["data"]["message"], "resource not found");
    }

    #[tokio::test]
    async fn account_crud_returns_direct_domain_values_and_null_units() {
        let (_directory, dispatcher) = test_dispatcher().await;
        let created = response(
            &dispatcher,
            r#"{
                "jsonrpc":"2.0",
                "id":1,
                "method":"account.create",
                "params":{"input":{
                    "account_identity":1,
                    "name":"Main",
                    "networks":[],
                    "metadata":{"type":"view","evm_address":"0x0000000000000000000000000000000000000000"},
                    "group_id":null,
                    "display_order":0
                }}
            }"#,
        )
        .await;

        assert_eq!(created["result"]["account_identity"], 1);
        assert_eq!(created["result"]["metadata"]["type"], "view");
        assert!(created["result"]["group_id"].is_null());

        let listed = response(
            &dispatcher,
            r#"{"jsonrpc":"2.0","id":2,"method":"account.list","params":{}}"#,
        )
        .await;
        assert_eq!(listed["result"].as_array().unwrap().len(), 1);

        let layout = response(
            &dispatcher,
            r#"{"jsonrpc":"2.0","id":3,"method":"account.layout.get","params":{}}"#,
        )
        .await;
        assert_eq!(layout["result"]["accounts"][0]["name"], "Main");

        let deleted = response(
            &dispatcher,
            r#"{"jsonrpc":"2.0","id":4,"method":"account.delete","params":{"account_identity":1}}"#,
        )
        .await;
        assert_eq!(deleted["result"], Value::Null);
    }

    #[test]
    fn optional_balance_fresh_flag_defaults_to_false() {
        let params = parse_params::<AccountBalancesParams>(json!({
            "account_identity": 1,
            "display_currency": "fiat:usd"
        }))
        .unwrap();

        assert!(!params.fresh);
    }
}
