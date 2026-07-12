use std::sync::Arc;

use sqlx::SqlitePool;
use tracing::info;

use crate::{
    config::Configuration,
    db::{SkipMigrations, connect},
    error::KoiError,
    models::{
        abi::AbiManager, account::balance_cache::BalanceCacheManager,
        connection::ConnectionManager, event::AppEventBus, network::manager::NetworkManager,
        quoter::man::QuoterManager, vendor::man::VendorManager,
        wallet_request::WalletRequestManager,
    },
};

pub type AppState = Arc<State>;

pub type DB = SqlitePool;

pub struct State {
    pub config: Configuration,
    pub database: SqlitePool,
    pub networks: NetworkManager,
    pub quoters: QuoterManager,
    pub balances: BalanceCacheManager,
    pub vendors: VendorManager,
    pub abis: AbiManager,
    pub connections: ConnectionManager,
    pub wallet_requests: WalletRequestManager,
    pub events: AppEventBus,
}

impl State {
    pub async fn new() -> Result<AppState, KoiError> {
        let config = Configuration::load()?;

        info!("Configuration: {:?}", config);

        let database = connect(&config.database_url, None).await?;
        let vendors = VendorManager::init(&database).await?;
        let networks = NetworkManager::default();
        let quoters = QuoterManager::init(&database).await?;
        let balances = BalanceCacheManager::new();
        let abis = AbiManager::new(config.abi_cache_dir.clone().into());
        let events = AppEventBus::new();
        let wallet_requests = WalletRequestManager::new(database.clone(), events.clone());
        let connections = ConnectionManager::new(events.clone(), wallet_requests.clone());

        Ok(Arc::new(State {
            networks,
            quoters,
            balances,
            vendors,
            abis,
            connections,
            wallet_requests,
            events,
            database,
            config,
        }))
    }

    pub async fn run_migrations(skip: SkipMigrations) -> Result<(), KoiError> {
        let config = Configuration::load()?;
        info!("Database: {}", config.database_url);
        connect(&config.database_url, skip).await?;
        Ok(())
    }
}
