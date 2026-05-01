use alloy::primitives::{Address, U256};
use tracing::info;
use super::Asset;
use crate::error::KoiError;
use crate::models::asset::identity::AssetIdentity;
use crate::state::AppState;
use crate::models::asset::metadata::AssetMetadataOption;
use alloy::sol;

sol! {
    #[sol(rpc)]
    contract ERC20 {
         function name() public view returns (string memory);
         function symbol() public view returns (string memory);
         function decimals() public view returns (uint8);
         function balanceOf(address owner) public view returns (uint256 balance);
    }
}

impl Asset {
    pub async fn fetch_erc20_metadata(
        state: &AppState,
        asset_identity: &AssetIdentity,
    ) -> Result<AssetMetadataOption, KoiError> {
        info!("Fetching metadata for asset {}", asset_identity);
        let (network_identity, address) = asset_identity.unwrap_address().ok_or(KoiError::Internal("Asset identity is not an ERC20 address".to_string()))?;
        let rpc = state.networks.get_pool(&network_identity).get_first_rpc().ok_or(KoiError::Internal("No RPC found for network".to_string()))?;
        let provider = rpc.get_provider().ok_or(KoiError::Internal("No provider found for RPC".to_string()))?;

        let contract = ERC20::new(address, provider);

        let token_name = contract.name().call().await.map_err(|e| KoiError::Internal(format!("Failed to fetch name: {}", e)))?;
        let token_symbol = contract.symbol().call().await.map_err(|e| KoiError::Internal(format!("Failed to fetch symbol: {}", e)))?;
        let token_decimals = contract.decimals().call().await.map_err(|e| KoiError::Internal(format!("Failed to fetch decimals: {}", e)))?;
        let token_logo_url = None;

        Ok(AssetMetadataOption {
            name: Some(token_name),
            symbol: Some(token_symbol),
            decimals: Some(token_decimals as u8),
            icon_url: token_logo_url,
        })
    }

    pub async fn fetch_erc20_balance(
        state: &AppState,
        asset_identity: &AssetIdentity,
        address: Address,
    ) -> Result<U256, KoiError> {
        info!("Fetching balance for asset {}", asset_identity);
        let (network_identity, token_address) = asset_identity.unwrap_address().ok_or(KoiError::Internal("Asset identity is not an ERC20 address".to_string()))?;
        let rpc = state.networks.get_pool(&network_identity).get_first_rpc().ok_or(KoiError::Internal("No RPC found for network".to_string()))?;
        let provider = rpc.get_provider().ok_or(KoiError::Internal("No provider found for RPC".to_string()))?;

        let contract = ERC20::new(token_address, provider);
        let balance = contract.balanceOf(address).call().await.map_err(|e| KoiError::Internal(format!("Failed to fetch balance: {}", e)))?;
        Ok(balance)
    }
}
