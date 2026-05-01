use crate::{
    error::KoiError,
    models::{
        account::Account,
        asset::{Asset, identity::AssetIdentity},
        network::identity::NetworkIdentity,
    },
    state::AppState,
};
use alloy::{
    primitives::{Address, U256},
    providers::Provider,
};

impl Asset {
    pub async fn fetch_balance(
        state: &AppState,
        asset_identity: &AssetIdentity,
        account: &Account,
    ) -> Result<U256, KoiError> {
        let address = account
            .metadata
            .unwrap_address()
            .ok_or(KoiError::Internal("Account has no address".to_string()))?;

        match asset_identity {
            AssetIdentity::Native(network_identity) => {
                if !account.networks.contains(network_identity) {
                    Err(KoiError::Internal(
                        "Account is not on the network".to_string(),
                    ))
                } else {
                    Asset::fetch_native_balance(state, address, network_identity).await
                }
            }
            AssetIdentity::ERC20(network_identity, _token_address) => {
                if !account.networks.contains(network_identity) {
                    Err(KoiError::Internal(
                        "Account is not on the network".to_string(),
                    ))
                } else {
                    Asset::fetch_erc20_balance(state, asset_identity, address.clone()).await
                }
            }
            AssetIdentity::Fiat(_code) => Err(KoiError::Internal(
                "Fiat balances are not supported".to_string(),
            )),
        }
    }

    pub async fn fetch_native_balance(
        state: &AppState,
        address: Address,
        network_identity: &NetworkIdentity,
    ) -> Result<U256, KoiError> {
        let rpc = state
            .networks
            .get_pool(network_identity)
            .get_first_rpc()
            .ok_or(KoiError::Internal("No RPC found for network".to_string()))?;
        let provider = rpc
            .get_provider()
            .ok_or(KoiError::Internal("No provider found for RPC".to_string()))?;

        let balance = provider
            .get_balance(address)
            .await
            .map_err(|e| KoiError::Internal(format!("Failed to fetch balance: {}", e)))?;
        Ok(balance)
    }
}
