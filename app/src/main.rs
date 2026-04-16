use dotenvy::dotenv;
use tracing::info;

use crate::state::State;

pub mod wallet;
pub mod http;
pub mod state;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    info!("Heya");

    let state = State::new();

    http::serve(state).await
}
