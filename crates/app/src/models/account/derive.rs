use std::str::FromStr;

use alloy::primitives::Address;
use alloy_signer_local::PrivateKeySigner;
use coins_bip32::path::DerivationPath;
use coins_bip32::prelude::XPriv;
use coins_bip39::{English, Mnemonic};

use crate::error::KoiError;

const DEFAULT_DERIVATION_PATH: &str = "m/44'/60'/0'/0/0";

pub fn default_derivation_path() -> &'static str {
    DEFAULT_DERIVATION_PATH
}

pub fn generate_mnemonic() -> Result<String, KoiError> {
    let mnemonic = Mnemonic::<English>::new_with_count(&mut rand::thread_rng(), 12)
        .map_err(|error| KoiError::Internal(format!("failed to generate mnemonic: {error}")))?;

    Ok(mnemonic.to_phrase())
}

pub fn derive_addresses_from_mnemonic(
    mnemonic: &str,
    paths: &[String],
) -> Result<Vec<(String, Address)>, KoiError> {
    let mnemonic = Mnemonic::<English>::new_from_phrase(mnemonic)
        .map_err(|error| KoiError::Internal(format!("invalid mnemonic: {error}")))?;
    let seed = mnemonic
        .to_seed(None)
        .map_err(|error| KoiError::Internal(format!("failed to compute seed: {error}")))?;

    let mut results = Vec::with_capacity(paths.len());

    for path in paths {
        let address = derive_address_from_seed(&seed, path)?;
        results.push((path.clone(), address));
    }

    Ok(results)
}

pub fn derive_address_from_private_key(private_key: &str) -> Result<Address, KoiError> {
    let cleaned = private_key
        .trim()
        .strip_prefix("0x")
        .unwrap_or(private_key.trim());
    let mut bytes = [0u8; 32];

    if cleaned.len() != 64 {
        return Err(KoiError::Internal(
            "private key must be 32 bytes (64 hex characters)".to_string(),
        ));
    }

    hex::decode_to_slice(cleaned, &mut bytes)
        .map_err(|error| KoiError::Internal(format!("invalid private key hex: {error}")))?;

    let signer = PrivateKeySigner::from_bytes(&bytes.into())
        .map_err(|error| KoiError::Internal(format!("invalid private key: {error}")))?;

    Ok(signer.address())
}

fn derive_address_from_seed(seed: &[u8], path: &str) -> Result<Address, KoiError> {
    let path = DerivationPath::from_str(path)
        .map_err(|error| KoiError::Internal(format!("invalid derivation path: {error}")))?;
    let root = XPriv::root_from_seed(seed, None)
        .map_err(|error| KoiError::Internal(format!("failed to create root key: {error}")))?;
    let derived = root
        .derive_path(&path)
        .map_err(|error| KoiError::Internal(format!("derivation failed: {error}")))?;
    let signing_key: &k256::ecdsa::SigningKey = derived.as_ref();
    let secret_bytes = signing_key.to_bytes();
    let bytes: [u8; 32] = secret_bytes.into();
    let signer = PrivateKeySigner::from_bytes(&bytes.into())
        .map_err(|error| KoiError::Internal(format!("derived invalid private key: {error}")))?;

    Ok(signer.address())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mnemonic_derivation_matches_expected_address() {
        // Well-known test vector.
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let paths = vec!["m/44'/60'/0'/0/0".to_string()];
        let results = derive_addresses_from_mnemonic(mnemonic, &paths).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0].1.to_string(),
            "0x9858EfFD232B4033E47d90003D41EC34EcaEda94"
        );
    }

    #[test]
    fn test_private_key_derivation() {
        // Well-known Ganache test account.
        let private_key = "0x4f3edf983ac636a65a842ce7c78d9aa706d3b113bce9c46f30d7d21715b23b1d";
        let result = derive_address_from_private_key(private_key);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().to_string(),
            "0x90F8bf6A479f320ead074411a4B0e7944Ea8c9C1"
        );
    }
}
