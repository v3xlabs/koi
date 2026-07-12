use std::{collections::HashMap, sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use openlv::OpenLvError;
use poem_openapi::{Enum, Object};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::{RwLock, oneshot};
use uuid::Uuid;

use crate::{
    error::KoiError,
    models::{
        account::{Account, identity::AccountIdentity},
        event::AppEventBus,
        network::identity::NetworkIdentity,
    },
    state::{AppState, DB},
};

mod assets;
mod permissions;
mod signing;

use assets::{Assets, WatchedAsset};
use permissions::{Permissions, requested_permissions, requested_permissions_response};

const APPROVAL_TIMEOUT: Duration = Duration::from_secs(55);
#[derive(Clone, Debug, Serialize, Deserialize, Object)]
pub struct FrontendWalletRequest {
    pub request_id: Uuid,
    pub connection_id: Uuid,
    pub kind: WalletRequestKind,
    pub method: String,
    pub params: Value,
    pub raw_request: Value,
    pub account_identity: AccountIdentity,
    pub network_identity: NetworkIdentity,
    pub account_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Enum)]
#[oai(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum WalletRequestKind {
    Permission,
    Asset,
    Signature,
    Transaction,
    Network,
    Read,
    Unknown,
}

struct PendingWalletRequest {
    request_id: Uuid,
    connection_id: Uuid,
    kind: WalletRequestKind,
    method: String,
    params: Value,
    raw_request: Value,
    account_identity: AccountIdentity,
    network_identity: NetworkIdentity,
    account_address: Option<String>,
    default_result: Value,
    approved_action: ApprovedWalletAction,
    created_at: DateTime<Utc>,
    reply: oneshot::Sender<Value>,
}

#[derive(Clone)]
pub struct WalletRequestManager {
    database: DB,
    pending: Arc<RwLock<HashMap<Uuid, PendingWalletRequest>>>,
    permissions: Permissions,
    assets: Assets,
    events: AppEventBus,
}

#[derive(Clone, Debug)]
enum ApprovedWalletAction {
    None,
    GrantPermissions(Vec<String>),
    WatchAsset(WatchedAsset),
}

impl WalletRequestManager {
    pub fn new(database: DB, events: AppEventBus) -> Self {
        Self {
            database,
            pending: Arc::new(RwLock::new(HashMap::new())),
            permissions: Permissions::default(),
            assets: Assets::default(),
            events,
        }
    }

    pub async fn all(&self) -> Vec<FrontendWalletRequest> {
        let mut requests = self
            .pending
            .read()
            .await
            .values()
            .map(FrontendWalletRequest::from)
            .collect::<Vec<_>>();

        requests.sort_by_key(|request| request.created_at);
        requests
    }

    pub async fn get(&self, request_id: Uuid) -> Result<FrontendWalletRequest, KoiError> {
        self.pending
            .read()
            .await
            .get(&request_id)
            .map(FrontendWalletRequest::from)
            .ok_or_else(|| KoiError::Internal(format!("wallet request not found: {request_id}")))
    }

    pub async fn approve(&self, request_id: Uuid) -> Result<FrontendWalletRequest, KoiError> {
        let request = self
            .pending
            .write()
            .await
            .remove(&request_id)
            .ok_or_else(|| KoiError::Internal(format!("wallet request not found: {request_id}")))?;
        let response = FrontendWalletRequest::from(&request);
        self.apply_approved_action(&request).await;

        let _ = request.reply.send(request.default_result);
        self.notify_changed();

        Ok(response)
    }

    pub async fn reject(
        &self,
        request_id: Uuid,
        message: Option<String>,
    ) -> Result<FrontendWalletRequest, KoiError> {
        let request = self
            .pending
            .write()
            .await
            .remove(&request_id)
            .ok_or_else(|| KoiError::Internal(format!("wallet request not found: {request_id}")))?;
        let response = FrontendWalletRequest::from(&request);
        let _ = request.reply.send(provider_error(
            4001,
            message.unwrap_or_else(|| "User rejected the request".to_string()),
        ));
        self.notify_changed();

        Ok(response)
    }

    pub async fn reject_connection(&self, connection_id: Uuid, message: impl Into<String>) {
        self.permissions.revoke(connection_id).await;

        let message = message.into();
        let request_ids = self
            .pending
            .read()
            .await
            .values()
            .filter(|request| request.connection_id == connection_id)
            .map(|request| request.request_id)
            .collect::<Vec<_>>();

        if request_ids.is_empty() {
            return;
        }

        let mut pending = self.pending.write().await;

        for request_id in request_ids {
            if let Some(request) = pending.remove(&request_id) {
                let _ = request.reply.send(provider_error(4001, message.clone()));
                self.notify_changed();
            }
        }
    }

    pub async fn handle_openlv_request(
        &self,
        state: &AppState,
        connection_id: Uuid,
        account_identity: AccountIdentity,
        network_identity: NetworkIdentity,
        raw_request: Value,
    ) -> Result<Value, OpenLvError> {
        if let Some(result) = self
            .immediate_response(
                state,
                connection_id,
                &account_identity,
                &network_identity,
                &raw_request,
            )
            .await
        {
            return Ok(result);
        }

        let method = request_method(&raw_request);
        if !is_approvable_method(&method) {
            return Ok(provider_error(
                4200,
                format!("Unsupported wallet method: {method}"),
            ));
        }

        let request_id = Uuid::new_v4();
        let params = raw_request
            .get("params")
            .cloned()
            .unwrap_or_else(|| Value::Array(Vec::new()));
        let account_address = self.account_address(&account_identity).await;
        let (default_result, approved_action) = self
            .approved_result(
                state,
                &account_identity,
                &network_identity,
                &method,
                &raw_request,
                account_address.as_deref(),
            )
            .await;
        let (reply, response) = oneshot::channel();

        let request = PendingWalletRequest {
            request_id,
            connection_id,
            kind: classify_method(&method),
            method,
            params,
            raw_request,
            account_identity,
            network_identity,
            account_address,
            default_result,
            approved_action,
            created_at: Utc::now(),
            reply,
        };

        self.pending.write().await.insert(request_id, request);
        self.notify_changed();

        match tokio::time::timeout(APPROVAL_TIMEOUT, response).await {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(_)) => Ok(provider_error(4001, "Request was cancelled")),
            Err(_) => {
                self.pending.write().await.remove(&request_id);
                self.notify_changed();
                Ok(provider_error(4001, "Request approval timed out"))
            }
        }
    }

    fn notify_changed(&self) {
        self.events.invalidate_route("/wallet-requests");
    }

    async fn immediate_response(
        &self,
        state: &AppState,
        connection_id: Uuid,
        account_identity: &AccountIdentity,
        network_identity: &NetworkIdentity,
        raw_request: &Value,
    ) -> Option<Value> {
        let method = request_method(raw_request);

        match method.as_str() {
            "eth_chainId" => Some(json!(format!("0x{:x}", network_identity.0))),
            "net_version" => Some(json!(network_identity.0.to_string())),
            "web3_clientVersion" => Some(json!("Koi/OpenLV")),
            "eth_accounts" => {
                if !self.permissions.has(connection_id, method.as_str()).await {
                    return Some(json!([]));
                }
                let accounts = self
                    .account_address(account_identity)
                    .await
                    .map(|address| vec![address])
                    .unwrap_or_default();

                Some(json!(accounts))
            }
            "wallet_getPermissions" => Some(self.permissions.list_response(connection_id).await),
            "wallet_getCapabilities" => Some(Assets::capabilities_response(network_identity)),
            "wallet_getAssets" => {
                if !self.permissions.has(connection_id, method.as_str()).await {
                    return None;
                }
                let account_address = self.account_address(account_identity).await;

                Some(
                    self.assets
                        .get(
                            state,
                            account_identity,
                            account_address.as_deref(),
                            raw_request,
                        )
                        .await
                        .unwrap_or_else(|error| provider_error(4001, error)),
                )
            }
            _ => None,
        }
    }

    async fn account_address(&self, account_identity: &AccountIdentity) -> Option<String> {
        Account::get_by_id(&self.database, account_identity.clone())
            .await
            .ok()
            .and_then(|account| account.metadata.unwrap_address())
            .map(|address| address.to_checksum(None))
    }

    async fn approved_result(
        &self,
        state: &AppState,
        account_identity: &AccountIdentity,
        network_identity: &NetworkIdentity,
        method: &str,
        raw_request: &Value,
        account_address: Option<&str>,
    ) -> (Value, ApprovedWalletAction) {
        match method {
            "eth_requestAccounts" => (
                account_address
                    .map(|address| json!([address]))
                    .unwrap_or_else(|| json!([])),
                ApprovedWalletAction::GrantPermissions(vec!["eth_accounts".to_string()]),
            ),
            "wallet_requestPermissions" => {
                let permissions = requested_permissions(raw_request);
                (
                    requested_permissions_response(&permissions),
                    ApprovedWalletAction::GrantPermissions(permissions),
                )
            }
            "wallet_getAssets" => {
                match self
                    .assets
                    .get(state, account_identity, account_address, raw_request)
                    .await
                {
                    Ok(result) => (
                        result,
                        ApprovedWalletAction::GrantPermissions(vec![
                            "wallet_getAssets".to_string(),
                        ]),
                    ),
                    Err(error) => (provider_error(4001, error), ApprovedWalletAction::None),
                }
            }
            "wallet_watchAsset" => {
                match assets::watched_asset(account_identity, network_identity, raw_request) {
                    Ok(watched_asset) => {
                        (json!(true), ApprovedWalletAction::WatchAsset(watched_asset))
                    }
                    Err(error) => (provider_error(4001, error), ApprovedWalletAction::None),
                }
            }
            "personal_sign" => (
                signing::sign_message(method, raw_request, account_address),
                ApprovedWalletAction::None,
            ),
            "eth_sign" => (
                signing::sign_message(method, raw_request, account_address),
                ApprovedWalletAction::None,
            ),
            _ => (
                provider_error(4200, format!("Unsupported wallet method: {method}")),
                ApprovedWalletAction::None,
            ),
        }
    }

    async fn apply_approved_action(&self, request: &PendingWalletRequest) {
        match &request.approved_action {
            ApprovedWalletAction::None => {}
            ApprovedWalletAction::GrantPermissions(methods) => {
                self.permissions.grant(request.connection_id, methods).await;
            }
            ApprovedWalletAction::WatchAsset(watched_asset) => {
                self.assets.watch(watched_asset).await;
            }
        }
    }
}

impl From<&PendingWalletRequest> for FrontendWalletRequest {
    fn from(request: &PendingWalletRequest) -> Self {
        FrontendWalletRequest {
            request_id: request.request_id,
            connection_id: request.connection_id,
            kind: request.kind,
            method: request.method.clone(),
            params: request.params.clone(),
            raw_request: request.raw_request.clone(),
            account_identity: request.account_identity.clone(),
            network_identity: request.network_identity.clone(),
            account_address: request.account_address.clone(),
            created_at: request.created_at,
        }
    }
}

fn request_method(request: &Value) -> String {
    request
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string()
}

fn classify_method(method: &str) -> WalletRequestKind {
    match method {
        "eth_requestAccounts" | "wallet_requestPermissions" => WalletRequestKind::Permission,
        "wallet_getAssets" | "wallet_watchAsset" => WalletRequestKind::Asset,
        "personal_sign"
        | "eth_sign"
        | "eth_signTypedData"
        | "eth_signTypedData_v3"
        | "eth_signTypedData_v4" => WalletRequestKind::Signature,
        "eth_sendTransaction" | "eth_signTransaction" => WalletRequestKind::Transaction,
        "wallet_switchEthereumChain" | "wallet_addEthereumChain" => WalletRequestKind::Network,
        "eth_accounts"
        | "eth_chainId"
        | "net_version"
        | "web3_clientVersion"
        | "eth_call"
        | "eth_getBalance"
        | "eth_getCode"
        | "eth_getTransactionCount"
        | "eth_estimateGas" => WalletRequestKind::Read,
        _ => WalletRequestKind::Unknown,
    }
}

fn is_approvable_method(method: &str) -> bool {
    matches!(
        method,
        "eth_requestAccounts"
            | "wallet_requestPermissions"
            | "wallet_getAssets"
            | "wallet_watchAsset"
            | "personal_sign"
            | "eth_sign"
    )
}

fn provider_error(code: i64, message: impl Into<String>) -> Value {
    json!({
        "error": {
            "code": code,
            "message": message.into(),
        },
    })
}
