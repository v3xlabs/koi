pub mod config;
pub mod db;
pub mod error;
pub mod http;
pub mod models;
pub mod state;
pub mod vendor;

use std::sync::Once;

pub const DEFAULT_API_URL: &str = "http://localhost:7777";

pub fn install_crypto_provider() {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    });
}
