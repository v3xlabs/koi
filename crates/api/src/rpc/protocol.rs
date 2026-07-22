//! JSON-RPC 2.0 envelopes, error objects, and protocol limits.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use ts_rs::TS;

pub const JSON_RPC_VERSION: &str = "2.0";
pub const MAX_MESSAGE_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_BATCH_ENTRIES: usize = 128;
pub const MAX_IN_FLIGHT_CALLS: usize = 128;

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
