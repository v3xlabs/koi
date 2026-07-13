use alloy::primitives::Address;
use poem::{Result, web::Data};
use poem_openapi::{Object, OpenApi, payload::Json};
use serde::{Deserialize, Serialize};

use crate::{
    http::auth::Auth,
    models::account::derive::{
        default_derivation_path, derive_address_from_private_key, derive_addresses_from_mnemonic,
        generate_mnemonic,
    },
    state::AppState,
};

use super::ApiTags;

pub struct AccountDeriveApi;

#[derive(Serialize, Deserialize, Object)]
pub struct DeriveMnemonicRequest {
    pub mnemonic: String,
    pub paths: Vec<String>,
}

#[derive(Serialize, Deserialize, Object)]
pub struct DeriveMnemonicResult {
    pub path: String,
    pub address: String,
}

#[derive(Serialize, Deserialize, Object)]
pub struct DeriveMnemonicResponse {
    pub results: Vec<DeriveMnemonicResult>,
}

#[derive(Serialize, Deserialize, Object)]
pub struct DerivePrivateKeyRequest {
    pub private_key: String,
}

#[derive(Serialize, Deserialize, Object)]
pub struct DeriveAddressResponse {
    pub address: String,
}

#[derive(Serialize, Deserialize, Object)]
pub struct GenerateMnemonicResponse {
    pub mnemonic: String,
}

#[derive(Serialize, Deserialize, Object)]
pub struct DefaultDerivationPathResponse {
    pub path: String,
}

fn format_address(address: Address) -> String {
    address.to_checksum(None)
}

#[OpenApi]
impl AccountDeriveApi {
    /// Generate a new BIP-39 mnemonic
    ///
    /// GET /api/acc/generate/mnemonic
    #[oai(
        path = "/acc/generate/mnemonic",
        method = "get",
        tag = "ApiTags::Account"
    )]
    async fn generate_mnemonic(
        &self,
        auth: Auth,
        _state: Data<&AppState>,
    ) -> Result<Json<GenerateMnemonicResponse>> {
        let _auth_data = auth.unwrap()?;
        let mnemonic = generate_mnemonic()?;

        Ok(Json(GenerateMnemonicResponse { mnemonic }))
    }

    /// Return the default BIP-44 Ethereum derivation path
    ///
    /// GET /api/acc/derive/default-path
    #[oai(
        path = "/acc/derive/default-path",
        method = "get",
        tag = "ApiTags::Account"
    )]
    async fn default_derivation_path(
        &self,
        auth: Auth,
        _state: Data<&AppState>,
    ) -> Result<Json<DefaultDerivationPathResponse>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(DefaultDerivationPathResponse {
            path: default_derivation_path().to_string(),
        }))
    }

    /// Derive addresses from a mnemonic and derivation paths
    ///
    /// POST /api/acc/derive/mnemonic
    #[oai(
        path = "/acc/derive/mnemonic",
        method = "post",
        tag = "ApiTags::Account"
    )]
    async fn derive_from_mnemonic(
        &self,
        auth: Auth,
        _state: Data<&AppState>,
        payload: Json<DeriveMnemonicRequest>,
    ) -> Result<Json<DeriveMnemonicResponse>> {
        let _auth_data = auth.unwrap()?;

        let results = derive_addresses_from_mnemonic(&payload.mnemonic, &payload.paths)?
            .into_iter()
            .map(|(path, address)| DeriveMnemonicResult {
                path,
                address: format_address(address),
            })
            .collect();

        Ok(Json(DeriveMnemonicResponse { results }))
    }

    /// Derive an address from a private key
    ///
    /// POST /api/acc/derive/private-key
    #[oai(
        path = "/acc/derive/private-key",
        method = "post",
        tag = "ApiTags::Account"
    )]
    async fn derive_from_private_key(
        &self,
        auth: Auth,
        _state: Data<&AppState>,
        payload: Json<DerivePrivateKeyRequest>,
    ) -> Result<Json<DeriveAddressResponse>> {
        let _auth_data = auth.unwrap()?;

        let address = derive_address_from_private_key(&payload.private_key)?;

        Ok(Json(DeriveAddressResponse {
            address: format_address(address),
        }))
    }
}
