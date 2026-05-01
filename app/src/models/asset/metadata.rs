use std::collections::HashMap;

use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::KoiError;
use crate::models::asset::identity::AssetIdentity;
use crate::state::AppState;
use crate::vendor::{avara, zerion};

use super::Asset;

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
}
