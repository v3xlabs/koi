use std::time::Instant;

use alloy::primitives::Address;
use sourcify::{
    Sourcify,
    v2::{self, Contract},
};
use tracing::info;

use crate::{error::KoiError, models::network::identity::NetworkIdentity};

pub async fn fetch_abi(
    network_identity: NetworkIdentity,
    address: Address,
) -> Result<Contract, KoiError> {
    let start = Instant::now();
    let x = Sourcify::default();
    let x = x.v2();
    let y = x
        .get_contract_with_fields(
            network_identity.0,
            address.to_string(),
            &[
                v2::field::ABI,
                v2::field::COMPILATION,
                v2::field::METADATA,
                "proxyResolution",
            ],
        )
        .await
        .map_err(|e| KoiError::Internal(format!("Failed to fetch contract from Sourcify: {}", e)))?
        .ok_or(KoiError::Internal("Contract not found".to_string()))?;

    info!("Sourcify fetch ABI took {}ms", start.elapsed().as_millis());
    Ok(y)
}
