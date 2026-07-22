use serde::{Deserialize, Serialize};
use tracing::warn;
use ts_rs::TS;

use crate::{
    error::KoiError,
    models::{
        asset::identity::AssetIdentity,
        quoter::discover::{
            erc4626::discover_erc4626_underlying_token,
            uniswap_v2::{UniswapV2Pair, discover_uniswap_v2_pair},
            uniswap_v3::{UniswapV3Pool, discover_uniswap_v3_pool},
        },
    },
    state::AppState,
};

pub mod erc4626;
pub mod uniswap_v2;
pub mod uniswap_v3;

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(optional_fields)]
pub struct QuoterDiscovery {
    pub token_a: AssetIdentity,
    pub token_b: Option<AssetIdentity>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(optional_fields)]
pub struct QuoterDiscoveryResponse {
    pub erc4626: Option<AssetIdentity>,
    pub uniswap_v2: Option<UniswapV2Pair>,
    pub uniswap_v3: Option<Vec<UniswapV3Pool>>,
}

impl QuoterDiscovery {
    pub async fn discover(&self, state: &AppState) -> Result<QuoterDiscoveryResponse, KoiError> {
        let erc4626 = discover_erc4626_underlying_token(state, &self.token_a)
            .await
            .ok();

        let uniswap_v2 = match &self.token_b {
            Some(token_b) => match discover_uniswap_v2_pair(state, &self.token_a, token_b).await {
                Ok(pair) => Some(pair),
                Err(e) => {
                    warn!("Failed to discover Uniswap V2 pair: {}", e);
                    None
                }
            },
            None => None,
        };

        let uniswap_v3 = match &self.token_b {
            Some(token_b) => match discover_uniswap_v3_pool(state, &self.token_a, token_b).await {
                Ok(pools) => Some(pools),
                Err(e) => {
                    warn!("Failed to discover Uniswap V3 pools: {}", e);
                    None
                }
            },
            None => None,
        };

        Ok(QuoterDiscoveryResponse {
            erc4626,
            uniswap_v2,
            uniswap_v3,
        })
    }
}
