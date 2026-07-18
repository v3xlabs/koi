use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::KoiError;
use crate::models::asset::identity::AssetIdentity;
use crate::models::vendor::flags::VendorFlag;
use crate::state::AppState;
use crate::vendor::{avara, blockscout, safe_wallet, smoldapp, zerion};

use super::Asset;

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(optional_fields)]
pub struct AssetMetadataOption {
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub decimals: Option<u8>,
    pub icon_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
pub struct AssetMetadataDiscovery {
    pub asset_identity: AssetIdentity,
    pub options: HashMap<String, AssetMetadataOption>,
}

impl Asset {
    pub async fn fetch_metadata(
        state: &AppState,
        asset_identity: &AssetIdentity,
    ) -> Result<AssetMetadataDiscovery, KoiError> {
        let mut options = HashMap::new();

        if let Ok(option) = Self::fetch_erc20_metadata(state, asset_identity).await {
            options.insert("erc20".to_string(), option);
        }

        let icon_options: Vec<Option<(Result<String, KoiError>, &'static str)>> = vec![
            match state.vendors.has_flag(VendorFlag::AvaraAssetIcons) {
                true => Some((avara::fetch_asset_icon(asset_identity).await, "avara")),
                false => None,
            },
            match state.vendors.has_flag(VendorFlag::ZerionAssetIcons) {
                true => Some((zerion::fetch_asset_icon(asset_identity).await, "zerion")),
                false => None,
            },
            match state.vendors.has_flag(VendorFlag::BlockscoutAssetIcons) {
                true => Some((
                    blockscout::fetch_asset_icon(asset_identity).await,
                    "blockscout",
                )),
                false => None,
            },
            match state.vendors.has_flag(VendorFlag::SmoldappAssetIcons) {
                true => Some((smoldapp::fetch_token_icon(asset_identity).await, "smoldapp")),
                false => None,
            },
            match state.vendors.has_flag(VendorFlag::SafewalletAssetIcons) {
                true => Some((
                    safe_wallet::fetch_asset_icon(asset_identity).await,
                    "safewallet",
                )),
                false => None,
            },
        ];

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
