//! Dispatches typed JSON-RPC requests to Koi's domain operations.

use alloy::primitives::U256;
use futures::{StreamExt, future::join_all, stream};
use koi::{
    error::KoiError,
    models::{
        account::{
            Account,
            derive::{
                default_derivation_path, derive_address_from_private_key,
                derive_addresses_from_mnemonic, generate_mnemonic,
            },
            group::AccountGroup,
            identity::AccountIdentity,
            layout::AccountLayout,
            metadata::WalletType,
        },
        asset::{Asset, identity::AssetIdentity},
        network::{Network, endpoint::NetworkEndpoint},
        quoter::Quoter,
        tx::{Tx, TxBase, decode::decode_transaction, simulate::simulate_transaction},
        vendor::flags::VendorFlag,
    },
    state::AppState,
    vendor::safe_wallet::tx::fetch_safewallet_tx,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value, json};

use super::*;

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
                    .get_balances(
                        &self.state,
                        &account,
                        &p.display_currency,
                        p.fresh.unwrap_or(false),
                    )
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

pub(crate) fn parse_params<T: DeserializeOwned>(params: Value) -> Result<T, RpcErrorObject> {
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
