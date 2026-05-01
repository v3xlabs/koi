// https://cdn.zerion.io/0x1abaea1f7c830bd89acc67ec4af516284b1bc33c.png
// check if an image is available at this url, if so proxy it.

use tracing::info;

use crate::{error::KoiError, models::asset::identity::AssetIdentity};

pub async fn fetch_icon_zerion(asset_identity: &AssetIdentity) -> Result<String, KoiError> {
    let (_network_identity, address) = asset_identity.unwrap_address().ok_or(KoiError::Internal("Asset identity is not an ERC20 address".to_string()))?;
    let url = format!("https://cdn.zerion.io/{}.png", format!("{:?}", address).to_lowercase());
    info!("Fetching icon from {}", url);
    let response = reqwest::get(&url).await.map_err(|e| KoiError::Internal(format!("Failed to fetch icon from Zerion: {}", e)))?;
    if response.status().is_success() {
        return Ok(url);
    }
    Err(KoiError::Internal("Failed to fetch icon from Zerion".to_string()))
}
