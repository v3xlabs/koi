use std::sync::Arc;

use figment::{Figment, providers::{Env, Serialized}};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool};
use tracing::info;

use crate::{error::KoiError, models::vendor::VendorManager};

pub type AppState = Arc<State>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Configuration {
    pub database_url: String,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            database_url: "sqlite://koi.db".to_string(),
        }
    }
}

pub struct State {
    pub database: SqlitePool,
    pub vendors: VendorManager,
    pub config: Configuration,
}

impl State {
    pub async fn new() -> Result<AppState, KoiError> {
        let config: Configuration = Figment::from(Serialized::defaults(Configuration::default()))
            .merge(Env::prefixed("KOI_"))
            .merge(Env::raw().only(&["DATABASE_URL"]))
            .extract()?;

        info!("Configuration: {:?}", config);

        let database = SqlitePool::connect(&config.database_url).await?;

        Ok(Arc::new(State {
            vendors: VendorManager::default(),
            database,
            config,
        }))
    }
}
