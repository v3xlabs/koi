use dotenvy::dotenv;
use tracing::info;

use crate::state::State;

pub mod error;
pub mod http;
pub mod models;
pub mod state;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    info!("Heya");

    let state = State::new().await.unwrap();

    http::serve(state).await
}
