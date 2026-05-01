use std::collections::HashMap;

use eth_prices::token::erc20;
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::KoiError;
use crate::models::asset::identity::AssetIdentity;
use crate::models::vendor::flags::VendorFlag;
use crate::state::AppState;
use crate::vendor::{avara, blockscout, zerion};

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

        let icon_options: Vec<Option<(Result<String, KoiError>, String)>> = vec![
            match state.vendors.has_flag(VendorFlag::AvaraTokenLogos) {
                true => Some((
                    avara::fetch_icon_avara(asset_identity).await,
                    "avara".to_string(),
                )),
                false => None,
            },
            match state.vendors.has_flag(VendorFlag::ZerionTokenLogos) {
                true => Some((
                    zerion::fetch_icon_zerion(asset_identity).await,
                    "zerion".to_string(),
                )),
                false => None,
            },
            match state.vendors.has_flag(VendorFlag::BlockscoutTokenLogos) {
                true => Some((
                    blockscout::fetch_icon_blockscout(asset_identity).await,
                    "blockscout".to_string(),
                )),
                false => None,
            },
        ];

        let mut options = HashMap::new();

        if let Ok(option) = erc20_option {
            options.insert("erc20".to_string(), option);
        }

        for option in icon_options {
            if let Some((Ok(url), name)) = option {
                options.insert(
                    name.to_string(),
                    AssetMetadataOption {
                        icon_url: Some(url),
                        name: None,
                        symbol: None,
                        decimals: None,
                    },
                );
            }
        }

        Ok(AssetMetadataDiscovery {
            asset_identity: asset_identity.clone(),
            options,
        })
    }
}
