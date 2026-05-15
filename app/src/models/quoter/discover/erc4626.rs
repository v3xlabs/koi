use eth_prices::quoter::erc4626::ERC4626;

use crate::{error::KoiError, models::asset::identity::AssetIdentity, state::AppState};

pub async fn discover_erc4626_underlying_token(
    state: &AppState,
    token_a: &AssetIdentity,
) -> Result<AssetIdentity, KoiError> {
    let (network_identity, address) = token_a
        .unwrap_address()
        .ok_or(KoiError::Internal("Token has no network".to_string()))?;
    let rpc = state
        .networks
        .get_pool(&network_identity)
        .get_first_rpc(state)
        .await?;
    let provider = rpc
        .get_provider()
        .ok_or(KoiError::Internal("No provider found for RPC".to_string()))?;

    let underlying_token = ERC4626::new(address, &provider)
        .asset()
        .call()
        .await
        .map_err(|e| {
            KoiError::Internal(format!(
                "Failed to discover ERC4626 underlying token: {}",
                e
            ))
        })?;
    Ok(AssetIdentity::ERC20(network_identity, underlying_token))
}
