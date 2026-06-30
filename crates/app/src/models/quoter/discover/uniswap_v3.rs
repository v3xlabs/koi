use crate::{
    error::KoiError,
    models::{
        asset::{erc20::ERC20, identity::AssetIdentity},
        network::identity::NetworkIdentity,
    },
    state::AppState,
};
use alloy::{
    primitives::{Address, address, aliases::U24},
    providers::DynProvider,
};
use eth_prices::quoter::uniswap_v3::discovery::UniswapV3Factory::{self, UniswapV3FactoryInstance};
use futures::{StreamExt, stream};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

fn factory_address(network_identity: &NetworkIdentity) -> Result<Address, KoiError> {
    match network_identity {
        NetworkIdentity(1) => Ok(address!("0x1F98431c8aD98523631AE4a59f267346ea31F984")),
        _ => Err(KoiError::Internal("Unsupported network".to_string())),
    }
}

fn fees(network_identity: &NetworkIdentity) -> Result<Vec<u32>, KoiError> {
    match network_identity {
        NetworkIdentity(1) => Ok(vec![500, 3000, 10000]),
        _ => Err(KoiError::Internal("Unsupported network".to_string())),
    }
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct UniswapV3Pool {
    pub pool_address: String,
    pub fee: u32,
    pub reserve_0: Option<String>,
    pub reserve_1: Option<String>,
}

async fn get_pool(
    factory: &UniswapV3FactoryInstance<&DynProvider>,
    token_a: &Address,
    token_b: &Address,
    fee: u32,
    provider: &DynProvider,
) -> Result<UniswapV3Pool, KoiError> {
    let (token_a, token_b) = if token_a < token_b {
        (*token_a, *token_b)
    } else {
        (*token_b, *token_a)
    };

    let pool_address = factory
        .getPool(token_a, token_b, U24::from(fee))
        .call()
        .await
        .map_err(|e| KoiError::Internal(format!("Failed to get pool: {}", e)))?;

    if pool_address.is_zero() {
        return Err(KoiError::Internal("Pool is not initialized".to_string()));
    }

    let reserve_0 = ERC20::new(token_a, provider)
        .balanceOf(pool_address)
        .call()
        .await
        .map_err(|e| KoiError::Internal(format!("Failed to get reserve 0: {}", e)))?;
    let reserve_1 = ERC20::new(token_b, provider)
        .balanceOf(pool_address)
        .call()
        .await
        .map_err(|e| KoiError::Internal(format!("Failed to get reserve 1: {}", e)))?;

    Ok(UniswapV3Pool {
        pool_address: pool_address.to_string(),
        fee,
        reserve_0: Some(reserve_0.to_string()),
        reserve_1: Some(reserve_1.to_string()),
    })
}

pub async fn discover_uniswap_v3_pool(
    state: &AppState,
    token_a: &AssetIdentity,
    token_b: &AssetIdentity,
) -> Result<Vec<UniswapV3Pool>, KoiError> {
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
    let factory = UniswapV3Factory::new(factory_address, &provider);

    let pools: Vec<Result<UniswapV3Pool, KoiError>> = stream::iter(fees(&network_identity)?)
        .map(async |fee| get_pool(&factory, &token_a, &token_b, fee, &provider).await)
        .buffer_unordered(8)
        .collect::<Vec<_>>()
        .await;

    let mut pools: Vec<UniswapV3Pool> = pools.into_iter().filter_map(|pool| pool.ok()).collect();

    pools.sort_by(|a, b| {
        let a_reserve_0 = a
            .reserve_0
            .clone()
            .map_or(0, |r| r.parse::<u128>().unwrap_or(0));
        let b_reserve_0 = b
            .reserve_0
            .clone()
            .map_or(0, |r| r.parse::<u128>().unwrap_or(0));
        b_reserve_0.cmp(&a_reserve_0)
    });

    Ok(pools)
}
