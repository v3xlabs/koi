//! JSON-RPC protocol envelopes and the typed method catalogue.

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

use super::*;

pub const JSON_RPC_VERSION: &str = "2.0";
pub const MAX_MESSAGE_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_BATCH_ENTRIES: usize = 128;
pub const MAX_IN_FLIGHT_CALLS: usize = 64;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS, PartialEq, Eq)]
pub enum JsonRpcVersion {
    #[serde(rename = "2.0")]
    V2,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[serde(untagged)]
pub enum RpcIdentity {
    Number(f64),
    String(String),
    Null(()),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RpcErrorKind {
    InvalidInput,
    NotFound,
    Conflict,
    Unavailable,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
pub struct RpcErrorData {
    pub kind: RpcErrorKind,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(optional_fields)]
pub struct RpcErrorObject {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<RpcErrorData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(optional_fields)]
pub struct RpcRequestEnvelope {
    pub jsonrpc: JsonRpcVersion,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RpcIdentity>,
    pub method: String,
    #[ts(type = "Record<string, unknown>")]
    pub params: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
pub struct RpcSuccessEnvelope {
    pub jsonrpc: JsonRpcVersion,
    pub id: RpcIdentity,
    #[ts(type = "unknown")]
    pub result: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
pub struct RpcErrorEnvelope {
    pub jsonrpc: JsonRpcVersion,
    pub id: RpcIdentity,
    pub error: RpcErrorObject,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[serde(untagged)]
pub enum RpcResponseEnvelope {
    Success(RpcSuccessEnvelope),
    Error(RpcErrorEnvelope),
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, Default)]
#[serde(deny_unknown_fields)]
pub struct EmptyParams {}

pub trait RpcMethod {
    type Params: Serialize + DeserializeOwned + TS + 'static;
    type Output: Serialize + DeserializeOwned + TS + 'static;
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

#[macro_export]
macro_rules! rpc_method_registry {
    ($callback:ident) => {
        $callback! {
            SystemPing, EmptyParams, String, "system.ping";
            AccountList, EmptyParams, Vec<Account>, "account.list";
            AccountGet, AccountParams, Account, "account.get";
            AccountCreate, AccountCreateParams, Account, "account.create";
            AccountNextIdentity, EmptyParams, AccountIdentity, "account.nextIdentity";
            AccountUpdate, AccountUpdateParams, Account, "account.update";
            AccountDelete, AccountParams, (), "account.delete";
            AccountAssetList, AccountParams, Vec<AssetIdentity>, "account.asset.list";
            AccountAssetAdd, AccountAssetParams, (), "account.asset.add";
            AccountAssetRemove, AccountAssetParams, (), "account.asset.remove";
            AccountAssetBalance, AccountAssetBalanceParams, AccountBalance, "account.asset.balance";
            AccountBalanceList, AccountBalancesParams, AccountBalances, "account.balance.list";
            AccountLayoutGet, EmptyParams, AccountLayout, "account.layout.get";
            AccountLayoutUpdate, LayoutUpdateParams, AccountLayout, "account.layout.update";
            AccountGroupCreate, GroupCreateParams, AccountGroup, "account.group.create";
            AccountGroupUpdate, GroupUpdateParams, AccountGroup, "account.group.update";
            AccountGroupDelete, GroupParams, (), "account.group.delete";
            AccountTransactionList, AccountParams, Vec<Tx>, "account.transaction.list";
            AccountTransactionPending, AccountParams, Vec<Tx>, "account.transaction.pending";
            AccountMnemonicGenerate, EmptyParams, String, "account.mnemonic.generate";
            AccountDerivationDefaultPath, EmptyParams, String, "account.derivation.defaultPath";
            AccountDerivationFromMnemonic, DeriveMnemonicParams, Vec<DeriveMnemonicResult>, "account.derivation.fromMnemonic";
            AccountDerivationFromPrivateKey, DerivePrivateKeyParams, String, "account.derivation.fromPrivateKey";
            AssetList, EmptyParams, Vec<Asset>, "asset.list";
            AssetGet, AssetParams, Asset, "asset.get";
            AssetCreate, AssetCreateParams, Asset, "asset.create";
            AssetUpdate, AssetUpdateParams, Asset, "asset.update";
            AssetDelete, AssetParams, (), "asset.delete";
            AssetDiscoverMetadata, AssetParams, AssetMetadataDiscovery, "asset.discoverMetadata";
            AssetQuote, AssetQuoteParams, String, "asset.quote";
            NetworkList, EmptyParams, Vec<Network>, "network.list";
            NetworkGet, NetworkParams, Network, "network.get";
            NetworkCreate, NetworkCreateParams, Network, "network.create";
            NetworkUpdate, NetworkUpdateParams, Network, "network.update";
            NetworkDelete, NetworkParams, (), "network.delete";
            NetworkListPresets, EmptyParams, Vec<Network>, "network.listPresets";
            NetworkDiscoverMetadata, NetworkParams, NetworkMetadataDiscovery, "network.discoverMetadata";
            NetworkRpcStats, NetworkParams, RpcPoolStats, "network.rpcStats";
            EndpointList, NetworkParams, Vec<NetworkEndpoint>, "network.endpoint.list";
            EndpointGet, EndpointParams, NetworkEndpoint, "network.endpoint.get";
            EndpointCreate, EndpointCreateParams, NetworkEndpoint, "network.endpoint.create";
            EndpointUpdate, EndpointUpdateParams, NetworkEndpoint, "network.endpoint.update";
            EndpointDelete, EndpointParams, (), "network.endpoint.delete";
            EndpointNextIdentity, NetworkParams, i32, "network.endpoint.nextIdentity";
            EndpointStatus, EndpointParams, RpcStatus, "network.endpoint.status";
            TransactionSimulate, SimulateParams, SimulateTransactionResponse, "transaction.simulate";
            TransactionDecode, DecodeParams, DecodeTransactionResponse, "transaction.decode";
            QuoterList, EmptyParams, Vec<Quoter>, "quoter.list";
            QuoterGet, QuoterParams, Quoter, "quoter.get";
            QuoterCreate, QuoterCreateParams, Quoter, "quoter.create";
            QuoterUpdate, QuoterUpdateParams, Quoter, "quoter.update";
            QuoterDiscover, QuoterDiscoverParams, QuoterDiscoveryResponse, "quoter.discover";
            VendorListEnabled, EmptyParams, Vec<VendorFlag>, "vendor.listEnabled";
            VendorListAll, EmptyParams, Vec<VendorFlagInfo>, "vendor.listAll";
            VendorEnable, VendorParams, (), "vendor.enable";
            VendorDisable, VendorParams, (), "vendor.disable";
        }
    };
}

pub mod methods {
    use super::*;

    macro_rules! define_rpc_methods {
        ($( $marker:ident, $params:ty, $output:ty, $name:literal; )*) => {
            $(method!($marker, $params, $output, $name);)*
        };
    }

    rpc_method_registry!(define_rpc_methods);
}
