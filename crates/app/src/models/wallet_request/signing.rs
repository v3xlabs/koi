use alloy::{
    hex,
    signers::{SignerSync, local::PrivateKeySigner},
};
use serde_json::{Value, json};

use super::provider_error;

// Demo-only signer for 0x954c70105D0448301E8c9E96501252C08A2457E1.
const DEMO_PRIVATE_KEY: &str = "0x0e32f7d963ec6bdc0cae2b2b3cc101664083548b522fc68c015bb780dca2a28f";

pub fn sign_message(method: &str, request: &Value, account_address: Option<&str>) -> Value {
    let signer = match DEMO_PRIVATE_KEY.parse::<PrivateKeySigner>() {
        Ok(signer) => signer,
        Err(error) => return provider_error(4001, format!("Invalid demo signing key: {error}")),
    };
    if account_address
        .is_none_or(|address| !address.eq_ignore_ascii_case(&signer.address().to_string()))
    {
        return provider_error(
            4100,
            "The selected account is not available for signing.",
        );
    }

    let message = match message(method, request) {
        Ok(message) => message,
        Err(error) => return provider_error(4001, error),
    };

    match signer.sign_message_sync(&message) {
        Ok(signature) => json!(signature.to_string()),
        Err(error) => provider_error(4001, format!("Failed to sign message: {error}")),
    }
}

fn message(method: &str, request: &Value) -> Result<Vec<u8>, String> {
    let index = match method {
        "personal_sign" => 0,
        "eth_sign" => 1,
        _ => return Err(format!("Unsupported signing method: {method}")),
    };
    let value = request
        .get("params")
        .and_then(Value::as_array)
        .and_then(|params| params.get(index))
        .ok_or_else(|| format!("{method} missing message parameter"))?;
    let message = value
        .as_str()
        .ok_or_else(|| "message parameter must be a string".to_string())?;

    if let Some(hex_message) = message.strip_prefix("0x") {
        hex::decode(hex_message).map_err(|error| format!("invalid hex message: {error}"))
    } else {
        Ok(message.as_bytes().to_vec())
    }
}
