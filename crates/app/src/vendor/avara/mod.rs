use tracing::info;

use crate::{error::KoiError, models::asset::identity::AssetIdentity};

pub async fn fetch_asset_icon(asset_identity: &AssetIdentity) -> Result<String, KoiError> {
    let (network_identity, address) = asset_identity.unwrap_address().ok_or(KoiError::Internal(
        "Asset identity is not an ERC20 address".to_string(),
    ))?;
    let url = format!(
        "https://token-logos.family.co/asset?id={}:{:?}",
        network_identity.0, address
    );
    info!("Fetching icon from {}", url);

    // if the url responds with 200, return the image url
    let response = reqwest::get(&url)
        .await
        .map_err(|e| KoiError::Internal(format!("Failed to fetch icon from {}: {}", url, e)))?;

    match response.status().is_success() {
        true => Ok(url),
        false => Err(KoiError::Internal(format!(
            "Failed to fetch icon from {}: {}",
            url,
            response.status()
        ))),
    }
}
