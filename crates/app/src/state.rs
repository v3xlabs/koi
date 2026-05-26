use std::sync::Arc;

use figment::{
    Figment,
    providers::{Env, Serialized},
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::info;

use crate::{
    error::KoiError,
    models::{
        abi::AbiManager,
        account::balance_cache::BalanceCacheManager,
        network::manager::NetworkManager,
        quoter::man::QuoterManager,
        vendor::man::VendorManager,
    },
};

pub type AppState = Arc<State>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Configuration {
    pub database_url: String,
    pub abi_cache_dir: String,
    pub rpc_requests_per_second: u32,
    pub rpc_max_in_flight_per_endpoint: usize,
    pub rpc_rate_limit_retries: u32,
    pub rpc_recent_sample_limit: usize,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            database_url: "sqlite://koi.db".to_string(),
            abi_cache_dir: "cache/abis".to_string(),
            rpc_requests_per_second: 12,
            rpc_max_in_flight_per_endpoint: 8,
            rpc_rate_limit_retries: 2,
            rpc_recent_sample_limit: 30,
        }
    }
}

pub type DB = SqlitePool;

pub struct State {
    pub config: Configuration,
    pub database: SqlitePool,
    pub networks: NetworkManager,
    pub quoters: QuoterManager,
    pub balances: BalanceCacheManager,
    pub vendors: VendorManager,
    pub abis: AbiManager,
}

impl State {
    pub async fn new() -> Result<AppState, KoiError> {
        let config: Configuration = Figment::from(Serialized::defaults(Configuration::default()))
            .merge(Env::prefixed("KOI_"))
            .merge(Env::raw().only(&["DATABASE_URL"]))
            .extract()?;

        info!("Configuration: {:?}", config);

        let database = SqlitePool::connect(&config.database_url).await?;

        let vendors = VendorManager::init(&database).await?;
        let networks = NetworkManager::default();
        let quoters = QuoterManager::init(&database).await?;
        let balances = BalanceCacheManager::new();
        let abis = AbiManager::new(config.abi_cache_dir.clone().into());

        Ok(Arc::new(State {
            networks,
            quoters,
            balances,
            vendors,
            abis,
            database,
            config,
        }))
    }
}
