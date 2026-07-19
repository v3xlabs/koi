//! Method-specific JSON-RPC parameter and result types.

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

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct AccountParams {
    pub account_identity: AccountIdentity,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct AssetParams {
    pub asset_identity: AssetIdentity,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct NetworkParams {
    pub network_identity: NetworkIdentity,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct AccountAssetParams {
    pub account_identity: AccountIdentity,
    pub asset_identity: AssetIdentity,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct AccountAssetBalanceParams {
    pub account_identity: AccountIdentity,
    pub asset_identity: AssetIdentity,
    pub display_currency: AssetIdentity,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(optional_fields)]
pub struct AccountBalancesParams {
    pub account_identity: AccountIdentity,
    pub display_currency: AssetIdentity,
    #[serde(default)]
    pub fresh: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct AccountCreateParams {
    pub input: Account,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct AccountUpdateParams {
    pub account_identity: AccountIdentity,
    pub input: AccountUpdate,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct LayoutUpdateParams {
    pub input: AccountLayoutUpdate,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct GroupCreateParams {
    pub input: AccountGroupCreate,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct GroupUpdateParams {
    pub group_identity: GroupIdentity,
    pub input: AccountGroupUpdate,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct GroupParams {
    pub group_identity: GroupIdentity,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct DeriveMnemonicInput {
    pub mnemonic: String,
    pub paths: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct DeriveMnemonicParams {
    pub input: DeriveMnemonicInput,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct DeriveMnemonicResult {
    pub path: String,
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct DerivePrivateKeyParams {
    pub input: String,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct AssetCreateParams {
    pub input: Asset,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct AssetUpdateParams {
    pub asset_identity: AssetIdentity,
    pub input: AssetUpdate,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(optional_fields)]
pub struct AssetQuoteParams {
    pub asset_identity: AssetIdentity,
    #[serde(default)]
    pub display_asset: Option<AssetIdentity>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct NetworkCreateParams {
    pub input: Network,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct NetworkUpdateParams {
    pub network_identity: NetworkIdentity,
    pub input: NetworkUpdate,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct EndpointParams {
    pub network_identity: NetworkIdentity,
    pub endpoint_identity: i32,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct EndpointCreateParams {
    pub network_identity: NetworkIdentity,
    pub input: NetworkEndpoint,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct EndpointUpdateParams {
    pub network_identity: NetworkIdentity,
    pub endpoint_identity: i32,
    pub input: NetworkEndpointUpdate,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct SimulateParams {
    pub network_identity: NetworkIdentity,
    pub input: SimulateTransactionRequest,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct DecodeParams {
    pub network_identity: NetworkIdentity,
    pub input: DecodeTransactionRequest,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct QuoterParams {
    pub quoter_identity: String,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct QuoterCreateParams {
    pub input: QuoterCreate,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct QuoterUpdateParams {
    pub quoter_identity: String,
    pub input: QuoterUpdate,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct QuoterDiscoverParams {
    pub input: QuoterDiscovery,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct VendorParams {
    pub flag: VendorFlag,
}
