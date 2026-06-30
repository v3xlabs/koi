use crate::{
    error::KoiError,
    models::{
        asset::{erc20::ERC20, identity::AssetIdentity},
        network::identity::NetworkIdentity,
    },
    state::AppState,
};
use alloy::{
    primitives::{Address, address},
    providers::DynProvider,
};
use eth_prices::quoter::uniswap_v2::discovery::UniswapV2Factory::{self, UniswapV2FactoryInstance};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

fn factory_address(network_identity: &NetworkIdentity) -> Result<Address, KoiError> {
    match network_identity {
        NetworkIdentity(1) => Ok(address!("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f")),
        _ => Err(KoiError::Internal("Unsupported network".to_string())),
    }
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct UniswapV2Pair {
    pub pair_address: String,
    pub reserve_0: Option<String>,
    pub reserve_1: Option<String>,
}

async fn get_pair(
    factory: &UniswapV2FactoryInstance<&DynProvider>,
    token_a: &Address,
    token_b: &Address,
    provider: &DynProvider,
) -> Result<UniswapV2Pair, KoiError> {
    let (token_a, token_b) = if token_a < token_b {
        (*token_a, *token_b)
    } else {
        (*token_b, *token_a)
    };

    let pair_address = factory
        .getPair(token_a, token_b)
        .call()
        .await
        .map_err(|e| KoiError::Internal(format!("Failed to get pair: {}", e)))?;

    if pair_address.is_zero() {
        return Err(KoiError::Internal("Pair is not initialized".to_string()));
    }

    let reserve_0 = ERC20::new(token_a, provider)
        .balanceOf(pair_address)
        .call()
        .await
        .map_err(|e| KoiError::Internal(format!("Failed to get reserve 0: {}", e)))?;
    let reserve_1 = ERC20::new(token_b, provider)
        .balanceOf(pair_address)
        .call()
        .await
        .map_err(|e| KoiError::Internal(format!("Failed to get reserve 1: {}", e)))?;

    Ok(UniswapV2Pair {
        pair_address: pair_address.to_string(),
        reserve_0: Some(reserve_0.to_string()),
        reserve_1: Some(reserve_1.to_string()),
    })
}

pub async fn discover_uniswap_v2_pair(
    state: &AppState,
    token_a: &AssetIdentity,
    token_b: &AssetIdentity,
) -> Result<UniswapV2Pair, KoiError> {
    let (network_identity, token_a) = token_a
        .unwrap_address()
        .ok_or(KoiError::Internal("Token has no address".to_string()))?;
    let (network_identity_b, token_b) = token_b
        .unwrap_address()
        .ok_or(KoiError::Internal("Token has no address".to_string()))?;

    if network_identity != network_identity_b {
        return Err(KoiError::Internal(
            "Tokens are on different networks".to_string(),
        ));
    }

    let rpc = state
        .networks
        .get_pool(&network_identity)
        .get_first_rpc(state)
        .await?;
    let provider = rpc
        .get_provider()
        .ok_or(KoiError::Internal("No provider found for RPC".to_string()))?;
    let factory_address = factory_address(&network_identity)?;
    let factory = UniswapV2Factory::new(factory_address, &provider);

    let pair = get_pair(&factory, &token_a, &token_b, &provider).await?;

    Ok(pair)
}
