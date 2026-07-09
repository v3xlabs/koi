use std::{collections::HashMap, sync::Arc, time::Duration};

use alloy::{
    hex,
    signers::{SignerSync, local::PrivateKeySigner},
};
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
    state::DB,
};

const APPROVAL_TIMEOUT: Duration = Duration::from_secs(55);
// Belongs to 0x954c70105D0448301E8c9E96501252C08A2457E1
const DEV_OPENLV_PRIVATE_KEY: &str =
    "0x0e32f7d963ec6bdc0cae2b2b3cc101664083548b522fc68c015bb780dca2a28f";

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
    created_at: DateTime<Utc>,
    reply: oneshot::Sender<Value>,
}

#[derive(Clone)]
pub struct WalletRequestManager {
    database: DB,
    pending: Arc<RwLock<HashMap<Uuid, PendingWalletRequest>>>,
    events: AppEventBus,
}

impl WalletRequestManager {
    pub fn new(database: DB, events: AppEventBus) -> Self {
        Self {
            database,
            pending: Arc::new(RwLock::new(HashMap::new())),
            events,
        }
    }

    pub async fn all(&self) -> Vec<FrontendWalletRequest> {
        let mut requests = self
            .pending
            .read()
            .await
            .values()
            .map(PendingWalletRequest::to_response)
            .collect::<Vec<_>>();

        requests.sort_by_key(|request| request.created_at);
        requests
    }

    pub async fn get(&self, request_id: Uuid) -> Result<FrontendWalletRequest, KoiError> {
        self.pending
            .read()
            .await
            .get(&request_id)
            .map(PendingWalletRequest::to_response)
            .ok_or_else(|| request_not_found(request_id))
    }

    pub async fn approve(&self, request_id: Uuid) -> Result<FrontendWalletRequest, KoiError> {
        let request = self.take(request_id).await?;
        let response = request.to_response();
        let _ = request.reply.send(request.default_result);
        self.notify_changed();

        Ok(response)
    }

    pub async fn reject(
        &self,
        request_id: Uuid,
        message: Option<String>,
    ) -> Result<FrontendWalletRequest, KoiError> {
        let request = self.take(request_id).await?;
        let response = request.to_response();
        let _ = request.reply.send(provider_error(
            4001,
            message.unwrap_or_else(|| "User rejected the request".to_string()),
        ));
        self.notify_changed();

        Ok(response)
    }

    pub async fn reject_connection(&self, connection_id: Uuid, message: impl Into<String>) {
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
        connection_id: Uuid,
        account_identity: AccountIdentity,
        network_identity: NetworkIdentity,
        raw_request: Value,
    ) -> Result<Value, OpenLvError> {
        if let Some(result) = self
            .immediate_response(&account_identity, &network_identity, &raw_request)
            .await
        {
            return Ok(result);
        }

        let request_id = Uuid::new_v4();
        let method = request_method(&raw_request);
        let params = raw_request
            .get("params")
            .cloned()
            .unwrap_or_else(|| Value::Array(Vec::new()));
        let account_address = self.account_address(&account_identity).await;
        let default_result =
            self.default_approved_result(&method, &raw_request, account_address.as_deref());
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

    async fn take(&self, request_id: Uuid) -> Result<PendingWalletRequest, KoiError> {
        self.pending
            .write()
            .await
            .remove(&request_id)
            .ok_or_else(|| request_not_found(request_id))
    }

    fn notify_changed(&self) {
        self.events.invalidate_route("/wallet-requests");
    }

    async fn immediate_response(
        &self,
        account_identity: &AccountIdentity,
        network_identity: &NetworkIdentity,
        raw_request: &Value,
    ) -> Option<Value> {
        match request_method(raw_request).as_str() {
            "eth_chainId" => Some(json!(format!("0x{:x}", network_identity.0))),
            "net_version" => Some(json!(network_identity.0.to_string())),
            "web3_clientVersion" => Some(json!("Koi/OpenLV")),
            "eth_accounts" => {
                let accounts = self
                    .account_address(account_identity)
                    .await
                    .map(|address| vec![address])
                    .unwrap_or_default();

                Some(json!(accounts))
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

    fn default_approved_result(
        &self,
        method: &str,
        raw_request: &Value,
        account_address: Option<&str>,
    ) -> Value {
        match method {
            "eth_requestAccounts" | "wallet_requestPermissions" => account_address
                .map(|address| json!([address]))
                .unwrap_or_else(|| json!([])),
            "personal_sign" => self.sign_message_result(personal_sign_message(raw_request)),
            "eth_sign" => self.sign_message_result(eth_sign_message(raw_request)),
            _ => provider_error(4200, format!("Unsupported wallet method: {method}")),
        }
    }

    fn sign_message_result(&self, message: Result<Vec<u8>, String>) -> Value {
        let message = match message {
            Ok(message) => message,
            Err(error) => return provider_error(4001, error),
        };
        let signer = match DEV_OPENLV_PRIVATE_KEY.parse::<PrivateKeySigner>() {
            Ok(signer) => signer,
            Err(error) => {
                return provider_error(4001, format!("Invalid OpenLV signing key: {error}"));
            }
        };

        match signer.sign_message_sync(&message) {
            Ok(signature) => json!(signature.to_string()),
            Err(error) => provider_error(4001, format!("Failed to sign message: {error}")),
        }
    }
}

impl PendingWalletRequest {
    fn to_response(&self) -> FrontendWalletRequest {
        FrontendWalletRequest {
            request_id: self.request_id,
            connection_id: self.connection_id,
            kind: self.kind,
            method: self.method.clone(),
            params: self.params.clone(),
            raw_request: self.raw_request.clone(),
            account_identity: self.account_identity.clone(),
            network_identity: self.network_identity.clone(),
            account_address: self.account_address.clone(),
            created_at: self.created_at,
        }
    }
}

fn request_not_found(request_id: Uuid) -> KoiError {
    KoiError::Internal(format!("wallet request not found: {request_id}"))
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

fn provider_error(code: i64, message: impl Into<String>) -> Value {
    json!({
        "error": {
            "code": code,
            "message": message.into(),
        },
    })
}

fn personal_sign_message(raw_request: &Value) -> Result<Vec<u8>, String> {
    request_params(raw_request)
        .and_then(|params| params.first())
        .ok_or_else(|| "personal_sign missing message parameter".to_string())
        .and_then(message_bytes)
}

fn eth_sign_message(raw_request: &Value) -> Result<Vec<u8>, String> {
    request_params(raw_request)
        .and_then(|params| params.get(1))
        .ok_or_else(|| "eth_sign missing message parameter".to_string())
        .and_then(message_bytes)
}

fn request_params(raw_request: &Value) -> Option<&Vec<Value>> {
    raw_request.get("params").and_then(Value::as_array)
}

fn message_bytes(value: &Value) -> Result<Vec<u8>, String> {
    let message = value
        .as_str()
        .ok_or_else(|| "message parameter must be a string".to_string())?;

    if let Some(hex_message) = message.strip_prefix("0x") {
        hex::decode(hex_message).map_err(|error| format!("invalid hex message: {error}"))
    } else {
        Ok(message.as_bytes().to_vec())
    }
}
