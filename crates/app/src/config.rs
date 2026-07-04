use std::path::{Path, PathBuf};

use figment::{
    Figment,
    providers::{Env, Serialized},
};
use serde::{Deserialize, Serialize};

use crate::error::KoiError;

const LOCAL_DB_NAME: &str = "koi.db";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Configuration {
    pub database_url: String,
    pub abi_cache_dir: String,
    pub image_cache_dir: String,
    pub rpc_requests_per_second: u32,
    pub rpc_max_in_flight_per_endpoint: usize,
    pub rpc_rate_limit_retries: u32,
    pub rpc_recent_sample_limit: usize,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            database_url: "auto".to_string(),
            abi_cache_dir: "cache/abis".to_string(),
            image_cache_dir: "auto".to_string(),
            rpc_requests_per_second: 12,
            rpc_max_in_flight_per_endpoint: 8,
            rpc_rate_limit_retries: 2,
            rpc_recent_sample_limit: 30,
        }
    }
}

impl Configuration {
    pub fn load() -> Result<Configuration, KoiError> {
        let mut config: Configuration =
            Figment::from(Serialized::defaults(Configuration::default()))
                .merge(Env::prefixed("KOI_"))
                .merge(Env::raw().only(&["DATABASE_URL"]))
                .extract()?;

        if config.database_url == "auto" {
            config.database_url = resolve_database_url()?;
        }
        if config.image_cache_dir == "auto" {
            config.image_cache_dir = resolve_cache_dir("images")?;
        }

        Ok(config)
    }
}

pub fn resolve_cache_dir(name: &str) -> Result<String, KoiError> {
    let cwd = std::env::current_dir().map_err(|error| {
        KoiError::Internal(format!("could not read current directory: {error}"))
    })?;
    Ok(resolve_cache_dir_in(&cwd, dirs::cache_dir(), name)
        .display()
        .to_string())
}

pub(crate) fn resolve_cache_dir_in(cwd: &Path, cache_dir: Option<PathBuf>, name: &str) -> PathBuf {
    cache_dir
        .unwrap_or_else(|| cwd.join(".cache"))
        .join("koi")
        .join(name)
}

pub fn resolve_database_url() -> Result<String, KoiError> {
    let cwd = std::env::current_dir().map_err(|error| {
        KoiError::Internal(format!("could not read current directory: {error}"))
    })?;
    resolve_database_url_in(&cwd, dirs::config_dir())
}

pub(crate) fn resolve_database_url_in(
    cwd: &Path,
    config_dir: Option<PathBuf>,
) -> Result<String, KoiError> {
    let local = cwd.join(LOCAL_DB_NAME);
    if local.is_file() {
        return Ok(format!("sqlite://{LOCAL_DB_NAME}"));
    }

    let base = config_dir
        .ok_or_else(|| KoiError::Internal("could not determine config directory".to_string()))?;
    let db_dir = base.join("koi");
    std::fs::create_dir_all(&db_dir).map_err(|error| {
        KoiError::Internal(format!("could not create koi config directory: {error}"))
    })?;
    let db_path = db_dir.join(LOCAL_DB_NAME);
    Ok(sqlite_url_for_path(&db_path))
}

fn sqlite_url_for_path(path: &Path) -> String {
    format!("sqlite://{}", path.display())
}
