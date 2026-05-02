use alloy::primitives::Address;
use tracing::info;

use crate::{error::KoiError, models::asset::identity::AssetIdentity};

fn token_icon_url(network_identity: u64, address: Address) -> String {
    format!(
        "https://raw.githubusercontent.com/SmolDapp/tokenAssets/main/tokens/{}/{}/logo.svg",
        network_identity,
        format!("{:?}", address).to_lowercase()
    )
}

fn network_icon_url(network_identity: u64) -> String {
    format!(
        "https://raw.githubusercontent.com/SmolDapp/tokenAssets/main/chains/{}/logo.svg",
        network_identity
    )
}

pub async fn fetch_network_icon(network_identity: u64) -> Result<String, KoiError> {
    let url = network_icon_url(network_identity);
    info!("Fetching icon from {}", url);

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

pub async fn fetch_token_icon(asset_identity: &AssetIdentity) -> Result<String, KoiError> {
    let (network_identity, address) = asset_identity.unwrap_address().ok_or(KoiError::Internal(
        "Asset identity is not an ERC20 address".to_string(),
    ))?;
    let url = token_icon_url(network_identity.0, address);
    info!("Fetching icon from {}", url);

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
