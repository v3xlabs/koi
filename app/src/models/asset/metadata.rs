use std::collections::HashMap;

use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::KoiError;
use crate::models::asset::identity::AssetIdentity;
use crate::state::AppState;
use crate::vendor::{avara, zerion};

use super::Asset;
use alloy::sol;

sol! {
    #[sol(rpc)]
    contract ERC20 {
         function name() public view returns (string memory);
         function symbol() public view returns (string memory);
         function decimals() public view returns (uint8);
    }
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct AssetMetadataOption {
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub decimals: Option<u8>,
    pub icon_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct AssetMetadataDiscovery {
    pub asset_identity: AssetIdentity,
    pub options: HashMap<String, AssetMetadataOption>,
}

impl Asset {
    pub async fn fetch_metadata(
        state: &AppState,
        asset_identity: &AssetIdentity,
    ) -> Result<AssetMetadataDiscovery, KoiError> {
        let erc20_option = Self::fetch_erc20_metadata(state, asset_identity).await;
        let avara_option = avara::fetch_icon_avara(asset_identity).await;
        let zerion_option = zerion::fetch_icon_zerion(asset_identity).await;

        let mut options = HashMap::new();

        if let Ok(option) = erc20_option {
            options.insert("erc20".to_string(), option);
        } else {
            return Err(KoiError::Internal("Failed to fetch ERC20 metadata".to_string()));
        }
        if let Ok(url) = avara_option {
            options.insert(
                "avara".to_string(),
                AssetMetadataOption {
                    name: None,
                    symbol: None,
                    decimals: None,
                    icon_url: Some(url),
                },
            );
        }
        if let Ok(url) = zerion_option {
            options.insert(
                "zerion".to_string(),
                AssetMetadataOption {
                    name: None,
                    symbol: None,
                    decimals: None,
                    icon_url: Some(url),
                },
            );
        }

        Ok(AssetMetadataDiscovery {
            asset_identity: asset_identity.clone(),
            options,
        })
    }

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
}
