use std::sync::Arc;

pub type AppState = Arc<State>;

pub struct State {
    // pub wallets: Vec<Wallet>,
}

impl State {
    pub fn new() -> AppState {
        Arc::new(State {
            // wallets: Vec::new(),
        })
    }
}
