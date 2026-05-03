use tracing::info;

use crate::{error::KoiError, models::asset::identity::AssetIdentity};

// discover chain ids through safe-api
// https://safe-client.safe.global/v1/chains/1/safes/0x123/balances/usd?trusted=false

// discover currencies and value according to safe-api
// https://safe-client.safe.global/v1/portfolio/0x123?fiatCode=usd&chainIds=1&trusted=false

// native token icon url
// https://safe-transaction-assets.safe.global/chains/{}/currency_logo.png

fn network_icon_url(network_identity: u64) -> String {
    format!(
        "https://safe-transaction-assets.safe.global/chains/{}/chain_logo.png",
        network_identity
    )
}

fn native_token_icon_url(network_identity: u64) -> String {
    format!(
        "https://safe-transaction-assets.safe.global/chains/{}/currency_logo.png",
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

pub async fn fetch_asset_icon(asset_identity: &AssetIdentity) -> Result<String, KoiError> {
    let network_identity = match asset_identity {
        AssetIdentity::Native(network_identity) => network_identity,
        _ => Err(KoiError::Internal(format!(
            "Asset identity is not a native token: {}",
            asset_identity
        )))?,
    };

    let url = native_token_icon_url(network_identity.0);
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
