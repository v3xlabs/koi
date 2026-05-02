use tracing::info;

use crate::{error::KoiError, models::asset::identity::AssetIdentity};

pub async fn fetch_asset_icon(asset_identity: &AssetIdentity) -> Result<String, KoiError> {
    let (_network_identity, address) = asset_identity.unwrap_address().ok_or(
        KoiError::Internal("Asset identity is not an ERC20 address".to_string()),
    )?;
    let url = format!(
        "https://cdn.zerion.io/{}.png",
        format!("{:?}", address).to_lowercase()
    );
    info!("Fetching icon from {}", url);
    let response = reqwest::get(&url)
        .await
        .map_err(|e| KoiError::Internal(format!("Failed to fetch icon from Zerion: {}", e)))?;
    if response.status().is_success() {
        return Ok(url);
    }
    Err(KoiError::Internal(
        "Failed to fetch icon from Zerion".to_string(),
    ))
}
