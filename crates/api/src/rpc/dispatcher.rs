//! JSON-RPC 2.0 message handling over the registered method records.
//!
//! Methods self-register into `koi::rpc::RPC_METHODS` at their definition
//! sites; the dispatcher only indexes that slice. Duplicate method names
//! panic when the map is first built.

use std::{collections::HashMap, sync::LazyLock};

use futures::future::join_all;
use koi::{
    error::KoiError,
    rpc::{MethodRecord, RPC_METHODS, RpcCallError},
    state::AppState,
};
use serde_json::{Map, Value, json};

use super::*;

static METHODS: LazyLock<HashMap<&'static str, &'static MethodRecord>> = LazyLock::new(|| {
    let mut map = HashMap::with_capacity(RPC_METHODS.len());
    for record in RPC_METHODS {
        assert!(
            map.insert(record.name, record).is_none(),
            "duplicate RPC method name: {}",
            record.name
        );
    }
    map
});

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
                invalid_request(&format!("batch exceeds {MAX_BATCH_ENTRIES} entries")),
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
        let Some(record) = METHODS.get(method) else {
            return Err(method_not_found());
        };
        if !params.is_object() {
            return Err(invalid_params());
        }

        (record.dispatch)(&self.state, params)
            .await
            .map_err(call_error)
    }
}

fn call_error(error: RpcCallError) -> RpcErrorObject {
    match error {
        RpcCallError::InvalidParams => invalid_params(),
        RpcCallError::Domain(error) => application_error(error),
        RpcCallError::Encode(error) => internal_error(error),
    }
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

fn method_not_found() -> RpcErrorObject {
    RpcErrorObject {
        code: -32601,
        message: "Method not found".to_string(),
        data: None,
    }
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
    if matches!(kind, RpcErrorKind::Internal) {
        tracing::error!(%error, "internal error while handling a JSON-RPC request");
    }
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
    tracing::error!(%error, "failed to encode a JSON-RPC value");
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
