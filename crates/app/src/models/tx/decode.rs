use alloy::dyn_abi::JsonAbiExt;
use alloy::{
    dyn_abi::DynSolValue,
    json_abi::{Function, JsonAbi, Param},
    primitives::{Address, Bytes, Selector, U256},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;

use crate::{
    error::KoiError,
    models::{
        alloy::{ApiAddress, ApiBytes, ApiU256},
        network::identity::NetworkIdentity,
    },
    state::AppState,
};

const MAX_NESTED_CALL_DEPTH: usize = 6;

#[derive(Debug, Serialize, Deserialize)]
pub struct DecodeTransactionRequest {
    #[serde(default)]
    pub from: Option<ApiAddress>,
    pub to: ApiAddress,
    #[serde(default)]
    pub value: Option<ApiU256>,
    #[serde(default)]
    pub data: Option<ApiBytes>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DecodeTransactionResponse {
    pub call: DecodedCall,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimulateTransactionRequest {
    #[serde(default)]
    pub from: Option<ApiAddress>,
    pub to: ApiAddress,
    #[serde(default)]
    pub value: Option<ApiU256>,
    #[serde(default)]
    pub data: Option<ApiBytes>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimulateTransactionResponse {
    pub call: DecodedCall,
}

impl From<&SimulateTransactionRequest> for DecodeTransactionRequest {
    fn from(request: &SimulateTransactionRequest) -> Self {
        Self {
            from: request.from,
            to: request.to,
            value: request.value,
            data: request.data.clone(),
        }
    }
}

/// High-level decoded call output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedCall {
    pub from: Option<ApiAddress>,
    pub to: ApiAddress,
    pub value: ApiU256,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    pub data: ApiBytes,
    pub selector: Option<ApiBytes>,
    pub decoded: Decoded,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub subcalls: Vec<DecodedCall>,
}

/// Best-effort decode result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Decoded {
    Verified(DecodedFunction),
    SignatureFallback(SignatureFallback),
    Raw(RawCall),
}

/// ABI-decoded function call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedFunction {
    pub contract: DecodedContract,
    pub selector: ApiBytes,
    pub function: String,
    pub signature: String,
    pub params: Vec<DecodedParam>,
}

/// Contract provenance used during decoding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedContract {
    pub address: ApiAddress,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<DecodedProxy>,
}

/// Proxy metadata when decoding through a verified implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedProxy {
    pub proxy_type: Option<String>,
    pub implementation: ApiAddress,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation_name: Option<String>,
}

/// One decoded input parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedParam {
    pub name: Option<String>,
    pub ty: String,
    pub value: Value,
}

/// Fallback when only 4byte resolution is available.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureFallback {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract: Option<DecodedContract>,
    pub selector: ApiBytes,
    pub candidates: Vec<String>,
}

/// Fallback when nothing better can be decoded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawCall {
    pub data: ApiBytes,
}

pub async fn decode_transaction(
    state: &AppState,
    network: &NetworkIdentity,
    request: &DecodeTransactionRequest,
) -> Result<DecodeTransactionResponse, KoiError> {
    let data = request.data.clone().map(|data| data.0).unwrap_or_default();

    let call = decode_call(
        state,
        network,
        request.from,
        request.to.0,
        request.value.map(|value| value.0).unwrap_or(U256::ZERO),
        data,
    )
    .await?;

    Ok(DecodeTransactionResponse { call })
}

async fn decode_call(
    state: &AppState,
    network: &NetworkIdentity,
    from: Option<ApiAddress>,
    to: Address,
    value: alloy::primitives::U256,
    data: Bytes,
) -> Result<DecodedCall, KoiError> {
    Box::pin(decode_call_at_depth(
        state, network, from, to, value, data, 0,
    ))
    .await
}

async fn decode_call_at_depth(
    state: &AppState,
    network: &NetworkIdentity,
    from: Option<ApiAddress>,
    to: Address,
    value: alloy::primitives::U256,
    data: Bytes,
    depth: usize,
) -> Result<DecodedCall, KoiError> {
    let selector = selector_from_data(&data);
    let decoded_call = match selector {
        Some(selector) => decode_verified_call(state, network, to, selector, &data).await?,
        None => Decoded::Raw(RawCall {
            data: data.clone().into(),
        }),
    };
    let subcalls = if depth >= MAX_NESTED_CALL_DEPTH {
        Vec::new()
    } else {
        decode_nested_calls(state, network, &decoded_call, depth + 1).await?
    };

    Ok(DecodedCall {
        from,
        to: to.into(),
        value: value.into(),
        operation: None,
        data: data.clone().into(),
        selector: selector.map(selector_to_api_bytes),
        decoded: decoded_call,
        subcalls,
    })
}

async fn decode_verified_call(
    state: &AppState,
    network: &NetworkIdentity,
    address: Address,
    selector: Selector,
    data: &[u8],
) -> Result<Decoded, KoiError> {
    let contract = match state.abis.fetch_contract(network, address).await {
        Ok(contract) => contract,
        Err(_) => {
            return Ok(Decoded::Raw(RawCall {
                data: Bytes::copy_from_slice(data).into(),
            }));
        }
    };

    let Some(abi_value) = contract.abi else {
        return Ok(Decoded::Raw(RawCall {
            data: Bytes::copy_from_slice(data).into(),
        }));
    };

    let abi: JsonAbi = serde_json::from_value(abi_value)
        .map_err(|err| KoiError::Internal(format!("failed to parse Sourcify ABI: {err}")))?;

    let proxy = decode_proxy_resolution(contract.proxy_resolution.as_ref())?;
    let (abi, contract_address, verified_name, proxy) = if let Some(proxy) = proxy {
        match state
            .abis
            .fetch_contract(network, proxy.implementation.0)
            .await
        {
            Ok(implementation) => {
                let implementation_abi = implementation.abi.ok_or_else(|| {
                    KoiError::Internal("Sourcify implementation ABI missing".to_string())
                })?;
                let implementation_abi: JsonAbi = serde_json::from_value(implementation_abi)
                    .map_err(|err| {
                        KoiError::Internal(format!(
                            "failed to parse Sourcify implementation ABI: {err}"
                        ))
                    })?;
                let verified_name = implementation
                    .compilation
                    .and_then(|compilation| compilation.name.or(compilation.fully_qualified_name))
                    .or(proxy.implementation_name.clone());

                (
                    implementation_abi,
                    proxy.implementation.0,
                    verified_name,
                    Some(proxy),
                )
            }
            Err(_) => {
                let verified_name = contract
                    .compilation
                    .and_then(|compilation| compilation.name.or(compilation.fully_qualified_name));
                (abi, address, verified_name, Some(proxy))
            }
        }
    } else {
        let verified_name = contract
            .compilation
            .and_then(|compilation| compilation.name.or(compilation.fully_qualified_name));
        (abi, address, verified_name, None)
    };

    let functions = abi
        .functions()
        .filter(|function| function.selector() == selector)
        .cloned()
        .collect::<Vec<_>>();

    if functions.is_empty() {
        return Ok(Decoded::SignatureFallback(SignatureFallback {
            contract: Some(DecodedContract {
                address: contract_address.into(),
                verified_name,
                proxy,
            }),
            selector: selector_to_api_bytes(selector),
            candidates: Vec::new(),
        }));
    }

    for function in &functions {
        let Ok(values) = decode_function_input(&function, data) else {
            continue;
        };

        return Ok(Decoded::Verified(DecodedFunction {
            contract: DecodedContract {
                address: contract_address.into(),
                verified_name: verified_name.clone(),
                proxy: proxy.clone(),
            },
            selector: selector_to_api_bytes(selector),
            function: function.name.clone(),
            signature: function.signature(),
            params: decoded_params(&function.inputs, &values),
        }));
    }

    Ok(Decoded::SignatureFallback(SignatureFallback {
        contract: Some(DecodedContract {
            address: contract_address.into(),
            verified_name,
            proxy,
        }),
        selector: selector_to_api_bytes(selector),
        candidates: functions
            .iter()
            .map(Function::signature)
            .collect::<Vec<_>>(),
    }))
}

fn decode_proxy_resolution(value: Option<&Value>) -> Result<Option<DecodedProxy>, KoiError> {
    let Some(value) = value else {
        return Ok(None);
    };

    if !value
        .get("isProxy")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Ok(None);
    }

    let Some(implementation) = value
        .get("implementations")
        .and_then(Value::as_array)
        .and_then(|implementations| implementations.first())
    else {
        return Ok(None);
    };

    let Some(address) = implementation.get("address").and_then(Value::as_str) else {
        return Ok(None);
    };

    let implementation_address = Address::from_str(address).map_err(|err| {
        KoiError::Internal(format!(
            "failed to parse proxy implementation address: {err}"
        ))
    })?;

    Ok(Some(DecodedProxy {
        proxy_type: value
            .get("proxyType")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        implementation: implementation_address.into(),
        implementation_name: implementation
            .get("name")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
    }))
}

async fn decode_nested_calls(
    state: &AppState,
    network: &NetworkIdentity,
    decoded: &Decoded,
    depth: usize,
) -> Result<Vec<DecodedCall>, KoiError> {
    let Decoded::Verified(decoded) = decoded else {
        return Ok(Vec::new());
    };

    let mut candidates = Vec::new();
    collect_param_nested_candidates(&decoded.params, &mut candidates);

    let mut calls = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        calls.push(
            Box::pin(decode_call_at_depth(
                state,
                network,
                None,
                candidate.to,
                candidate.value,
                candidate.data,
                depth,
            ))
            .await
            .map(|mut call| {
                call.operation = Some(candidate.operation);
                call
            })?,
        );
    }

    Ok(calls)
}

struct NestedCallCandidate {
    to: Address,
    value: U256,
    operation: String,
    data: Bytes,
}

fn collect_param_nested_candidates(
    params: &[DecodedParam],
    candidates: &mut Vec<NestedCallCandidate>,
) {
    if let Some(transactions) = multi_send_transactions(params) {
        candidates.extend(transactions);
        return;
    }

    let to = find_address_param(params, "to");
    let value = find_u256_param(params, "value").unwrap_or(U256::ZERO);
    let data = find_bytes_param(params, "data");

    if let (Some(to), Some(data)) = (to, data) {
        if looks_like_calldata(&data) {
            candidates.push(NestedCallCandidate {
                to,
                value,
                operation: "call".to_string(),
                data,
            });
            return;
        }
    }

    for param in params {
        collect_value_nested_candidates(&param.value, candidates);
    }
}

fn collect_value_nested_candidates(value: &Value, candidates: &mut Vec<NestedCallCandidate>) {
    match value {
        Value::Array(values) => {
            if let Some(candidate) = nested_candidate_from_values(values) {
                candidates.push(candidate);
                return;
            }

            for value in values {
                collect_value_nested_candidates(value, candidates);
            }
        }
        Value::Object(values) => {
            let values = values.values().cloned().collect::<Vec<_>>();
            if let Some(candidate) = nested_candidate_from_values(&values) {
                candidates.push(candidate);
                return;
            }

            for value in values {
                collect_value_nested_candidates(&value, candidates);
            }
        }
        _ => {}
    }
}

fn nested_candidate_from_values(values: &[Value]) -> Option<NestedCallCandidate> {
    if let Some(candidate) = multicall_bundle_candidate(values) {
        return Some(candidate);
    }

    let to = values.iter().find_map(value_as_address)?;
    let data = values.iter().find_map(value_as_calldata)?;

    Some(NestedCallCandidate {
        to,
        value: U256::ZERO,
        operation: "call".to_string(),
        data,
    })
}

fn multicall_bundle_candidate(values: &[Value]) -> Option<NestedCallCandidate> {
    let to = values.first().and_then(value_as_address)?;
    let data = values.get(1).and_then(value_as_calldata)?;
    let value = values.get(2).and_then(value_as_u256).unwrap_or(U256::ZERO);

    Some(NestedCallCandidate {
        to,
        value,
        operation: "call".to_string(),
        data,
    })
}

fn multi_send_transactions(params: &[DecodedParam]) -> Option<Vec<NestedCallCandidate>> {
    let transactions = params
        .iter()
        .find(|param| param.name.as_deref() == Some("transactions"))
        .and_then(|param| value_as_bytes(&param.value))?;

    parse_multi_send_transactions(&transactions)
}

fn parse_multi_send_transactions(transactions: &[u8]) -> Option<Vec<NestedCallCandidate>> {
    let mut offset = 0;
    let mut calls = Vec::new();

    while offset < transactions.len() {
        let operation = *transactions.get(offset)?;
        offset += 1;

        let to = Address::from_slice(transactions.get(offset..offset + 20)?);
        offset += 20;

        let value = U256::from_be_slice(transactions.get(offset..offset + 32)?);
        offset += 32;

        let data_len = U256::from_be_slice(transactions.get(offset..offset + 32)?);
        offset += 32;

        let data_len: usize = data_len.try_into().ok()?;
        let data = Bytes::copy_from_slice(transactions.get(offset..offset + data_len)?);
        offset += data_len;

        calls.push(NestedCallCandidate {
            to,
            value,
            operation: match operation {
                0 => "call",
                1 => "delegate_call",
                _ => "unknown",
            }
            .to_string(),
            data,
        });
    }

    Some(calls)
}

fn find_address_param(params: &[DecodedParam], name: &str) -> Option<Address> {
    params
        .iter()
        .find(|param| param.name.as_deref() == Some(name))
        .and_then(|param| value_as_address(&param.value))
}

fn find_u256_param(params: &[DecodedParam], name: &str) -> Option<U256> {
    params
        .iter()
        .find(|param| param.name.as_deref() == Some(name))
        .and_then(|param| value_as_u256(&param.value))
}

fn find_bytes_param(params: &[DecodedParam], name: &str) -> Option<Bytes> {
    params
        .iter()
        .find(|param| param.name.as_deref() == Some(name))
        .and_then(|param| value_as_calldata(&param.value))
}

fn value_as_address(value: &Value) -> Option<Address> {
    let value = value.as_str()?;
    Address::from_str(value).ok()
}

fn value_as_u256(value: &Value) -> Option<U256> {
    let value = value.as_str()?;
    U256::from_str(value).ok()
}

fn value_as_calldata(value: &Value) -> Option<Bytes> {
    let bytes = value_as_bytes(value)?;
    looks_like_calldata(&bytes).then_some(bytes)
}

fn value_as_bytes(value: &Value) -> Option<Bytes> {
    let value = value.as_str()?;
    Bytes::from_str(value).ok()
}

fn decode_function_input(function: &Function, data: &[u8]) -> Result<Vec<DynSolValue>, KoiError> {
    function
        .abi_decode_input(&data[4..])
        .map_err(|err| KoiError::Internal(format!("failed to ABI decode input: {err}")))
}

fn decoded_params(params: &[Param], values: &[DynSolValue]) -> Vec<DecodedParam> {
    params
        .iter()
        .zip(values)
        .map(|(param, value)| DecodedParam {
            name: (!param.name.is_empty()).then(|| param.name.clone()),
            ty: param.ty.clone(),
            value: dyn_value_to_json(value),
        })
        .collect()
}

fn dyn_value_to_json(value: &DynSolValue) -> Value {
    match value {
        DynSolValue::Bool(value) => Value::Bool(*value),
        DynSolValue::Int(value, _) => Value::String(value.to_string()),
        DynSolValue::Uint(value, _) => Value::String(value.to_string()),
        DynSolValue::FixedBytes(value, size) => {
            Value::String(format!("0x{}", hex::encode(&value[..*size])))
        }
        DynSolValue::Address(value) => Value::String(value.to_string()),
        DynSolValue::Function(value) => Value::String(format!("0x{}", hex::encode(value))),
        DynSolValue::Bytes(value) => Value::String(format!("0x{}", hex::encode(value))),
        DynSolValue::String(value) => Value::String(value.clone()),
        DynSolValue::Array(values)
        | DynSolValue::FixedArray(values)
        | DynSolValue::Tuple(values) => {
            Value::Array(values.iter().map(dyn_value_to_json).collect())
        }
    }
}

fn selector_from_data(data: &[u8]) -> Option<Selector> {
    data.get(..4).map(Selector::from_slice)
}

fn selector_to_api_bytes(selector: Selector) -> ApiBytes {
    Bytes::copy_from_slice(selector.as_slice()).into()
}

fn looks_like_calldata(data: &[u8]) -> bool {
    data.len() >= 4
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn multicall_bundle_candidate_uses_tuple_positions() {
        let values = [
            json!("0x4A6c312ec70E8747a587EE860a0353cd42Be0aE0"),
            json!("0xd96ca0b9000000000000000000000000a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"),
            json!("123"),
            json!(false),
            json!("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ];

        let candidate = nested_candidate_from_values(&values).expect("candidate");

        assert_eq!(
            candidate.to,
            Address::from_str("0x4A6c312ec70E8747a587EE860a0353cd42Be0aE0").unwrap()
        );
        assert_eq!(candidate.data.as_ref()[..4], [0xd9, 0x6c, 0xa0, 0xb9]);
        assert_eq!(candidate.value, U256::from(123));
    }
}
