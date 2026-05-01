use tracing::info;

use crate::{error::KoiError, models::asset::identity::AssetIdentity};

/// Fetch the icon from the token-logos.family.co/asset?id=1:0x endpoint
/// This requires the `avara_token_logos` vendor flag
pub async fn fetch_icon_avara(asset_identity: &AssetIdentity) -> Result<String, KoiError> {
    // if !state.vendors.has_flag("avara_token_logos") {
    //     return Err(KoiError::Internal(
    //         "Avara token logos vendor flag is not enabled".to_string(),
    //     ));
    // }

    // do work
    let (network_identity, address) = asset_identity.unwrap_address().ok_or(KoiError::Internal("Asset identity is not an ERC20 address".to_string()))?;
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
