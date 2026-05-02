use std::{
    collections::HashMap,
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use alloy::primitives::Address;
use sourcify::v2::Contract;
use tokio::{fs, sync::Notify};
use tracing::{debug, warn};

use crate::{
    error::KoiError, models::network::identity::NetworkIdentity, vendor::sourcify::fetch_abi,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AbiCacheKey {
    network_identity: u64,
    address: String,
}

pub struct AbiManager {
    cache_dir: PathBuf,
    in_flight: Mutex<HashMap<AbiCacheKey, Arc<Notify>>>,
}

impl AbiManager {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            in_flight: Mutex::new(HashMap::new()),
        }
    }

    pub async fn fetch_contract(
        &self,
        network_identity: &NetworkIdentity,
        address: Address,
    ) -> Result<Contract, KoiError> {
        let key = AbiCacheKey {
            network_identity: network_identity.0,
            address: address.to_string().to_lowercase(),
        };

        loop {
            match self.read_cached_contract(&key).await {
                Ok(Some(contract)) => {
                    debug!(
                        network_identity = key.network_identity,
                        address = key.address,
                        "ABI cache hit"
                    );
                    return Ok(contract);
                }
                Ok(None) => {}
                Err(err) => {
                    warn!(
                        network_identity = key.network_identity,
                        address = key.address,
                        error = %err,
                        "Failed to read cached ABI; refetching"
                    );
                }
            }

            let (notify, is_leader) = {
                let mut in_flight = self.in_flight.lock().expect("ABI mutex poisoned");
                if let Some(notify) = in_flight.get(&key) {
                    (notify.clone(), false)
                } else {
                    let notify = Arc::new(Notify::new());
                    in_flight.insert(key.clone(), notify.clone());
                    (notify, true)
                }
            };

            if !is_leader {
                notify.notified().await;
                continue;
            }

            let result = self
                .fetch_and_cache_contract(network_identity.clone(), address, &key)
                .await;

            let notify = self
                .in_flight
                .lock()
                .expect("ABI mutex poisoned")
                .remove(&key);
            if let Some(notify) = notify {
                notify.notify_waiters();
            }

            return result;
        }
    }

    async fn fetch_and_cache_contract(
        &self,
        network_identity: NetworkIdentity,
        address: Address,
        key: &AbiCacheKey,
    ) -> Result<Contract, KoiError> {
        let contract = fetch_abi(network_identity, address).await?;
        if let Err(err) = self.write_cached_contract(key, &contract).await {
            warn!(
                network_identity = key.network_identity,
                address = key.address,
                error = %err,
                "Failed to write ABI cache"
            );
        }
        Ok(contract)
    }

    async fn read_cached_contract(&self, key: &AbiCacheKey) -> Result<Option<Contract>, KoiError> {
        let path = self.cache_path(key);
        let bytes = match fs::read(&path).await {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == ErrorKind::NotFound => return Ok(None),
            Err(err) => {
                return Err(KoiError::Internal(format!(
                    "failed to read ABI cache file {}: {err}",
                    path.display()
                )));
            }
        };

        serde_json::from_slice(&bytes).map(Some).map_err(|err| {
            KoiError::Internal(format!(
                "failed to parse ABI cache file {}: {err}",
                path.display()
            ))
        })
    }

    async fn write_cached_contract(
        &self,
        key: &AbiCacheKey,
        contract: &Contract,
    ) -> Result<(), KoiError> {
        let path = self.cache_path(key);
        let parent = path.parent().ok_or_else(|| {
            KoiError::Internal(format!("ABI cache path has no parent: {}", path.display()))
        })?;

        fs::create_dir_all(parent).await.map_err(|err| {
            KoiError::Internal(format!(
                "failed to create ABI cache directory {}: {err}",
                parent.display()
            ))
        })?;

        let temp_path = temp_path(parent, &path)?;
        let bytes = serde_json::to_vec(contract)
            .map_err(|err| KoiError::Internal(format!("failed to serialize ABI cache: {err}")))?;

        fs::write(&temp_path, bytes).await.map_err(|err| {
            KoiError::Internal(format!(
                "failed to write ABI cache file {}: {err}",
                temp_path.display()
            ))
        })?;

        fs::rename(&temp_path, &path).await.map_err(|err| {
            KoiError::Internal(format!(
                "failed to move ABI cache file {} to {}: {err}",
                temp_path.display(),
                path.display()
            ))
        })?;

        Ok(())
    }

    fn cache_path(&self, key: &AbiCacheKey) -> PathBuf {
        self.cache_dir
            .join("sourcify")
            .join(key.network_identity.to_string())
            .join(format!("{}.json", key.address))
    }
}

fn temp_path(parent: &Path, path: &Path) -> Result<PathBuf, KoiError> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            KoiError::Internal(format!(
                "ABI cache path has no file name: {}",
                path.display()
            ))
        })?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    Ok(parent.join(format!(".{file_name}.{}.{}.tmp", std::process::id(), nonce)))
}
