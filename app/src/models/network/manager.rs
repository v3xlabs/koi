use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::models::network::{identity::NetworkIdentity, pool::RpcPool};

/// Manages the RPC clients for all network endpoints
/// Scoped by network
/// When asked for a network rpc it will create a new client if it doesn't exist, but only if endpoint is enabled
/// when endpoints are enabled or disabled the enable() or disable() method should be called and we should shutdown the client and remove it
/// This struct is accessible via AppState
#[derive(Default)]
pub struct NetworkManager {
    clients: Mutex<HashMap<NetworkIdentity, Arc<RpcPool>>>,
}

impl NetworkManager {
    pub fn get_pool(&self, network_id: &NetworkIdentity) -> Arc<RpcPool> {
        let mut pools = self.clients.lock().unwrap();

        let pool = pools.get(network_id);
        if let Some(pool) = pool {
            return pool.clone();
        }

        let pool = Arc::new(RpcPool::new(network_id.clone()));
        pools.insert(network_id.clone(), pool.clone());
        pool
    }
}
