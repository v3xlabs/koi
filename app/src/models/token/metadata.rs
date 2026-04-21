use crate::{error::KoiError, state::AppState};

use super::Token;

impl Token {
    pub async fn fetch_metadata() {}

    /// Fetch the icon from the token-logos.family.co/asset?id=1:0x endpoint
    /// This requires the `avara_token_logos` vendor flag
    pub async fn fetch_icon_avara(state: &AppState) -> Result<String, KoiError> {
        if !state.vendors.has_flag("avara_token_logos") {
            return Err(KoiError::Internal(
                "Avara token logos vendor flag is not enabled".to_string(),
            ));
        }

        // do work
        let network_id = 1;
        let address = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";
        let url = format!(
            "https://token-logos.family.co/asset?id={}:{}",
            network_id, address
        );

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
}
