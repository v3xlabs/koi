use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{error::KoiError, models::asset::identity::AssetIdentity};

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockscoutMetadata {
    pub icon_url: String,
    // pub decimals: String,
    // pub name: String,
    // pub symbol: String,

    #[serde(flatten)]
    pub other: HashMap<String, String>,
}

/// https://eth.blockscout.com/api/v2/tokens/0x123
pub async fn fetch_icon_blockscout(asset_identity: &AssetIdentity) -> Result<String, KoiError> {
    let (_network_identity, address) = asset_identity.unwrap_address().ok_or(KoiError::Internal(
        "Asset identity is not an ERC20 address".to_string(),
    ))?;
    let url = format!(
        "https://eth.blockscout.com/api/v2/tokens/{}",
        address,
    );
    info!("Fetching icon from {}", url);
    let response = reqwest::get(&url)
        .await
        .map_err(|e| KoiError::Internal(format!("Failed to fetch icon from Blockscout: {}", e)))?;

    if response.status().is_success() {
        let metadata: BlockscoutMetadata = response.json().await.map_err(|e| KoiError::Internal(format!("Failed to parse Blockscout metadata: {}", e)))?;

        // TODO: Handle the other metadata

        return Ok(metadata.icon_url);
    }
    Err(KoiError::Internal(
        "Failed to fetch icon from Blockscout".to_string(),
    ))
}
