use std::sync::Arc;

use crate::models::vendor::VendorManager;

pub type AppState = Arc<State>;

pub struct State {
    // pub wallets: Vec<Wallet>,
    pub vendors: VendorManager,
}

impl State {
    pub fn new() -> AppState {
        Arc::new(State {
            vendors: VendorManager::default(),
            // wallets: Vec::new(),
        })
    }
}
