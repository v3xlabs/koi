//! Bridge API surface for the Flutter app.
//!
//! Mobile uses the same typed dispatcher as the WebSocket daemon, but calls it
//! directly without transport authentication.

use koi::state::State;
use koi_api::{
    Dispatcher,
    rpc::{EmptyParams, methods::SystemPing},
};

pub struct InProcessClient {
    dispatcher: Dispatcher,
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}

pub async fn create_client() -> Result<InProcessClient, String> {
    let state = State::new().await.map_err(|error| error.safe_message())?;
    Ok(InProcessClient {
        dispatcher: Dispatcher::new(state),
    })
}

pub async fn system_ping(client: &InProcessClient) -> Result<String, String> {
    client
        .dispatcher
        .call::<SystemPing>(EmptyParams::default())
        .await
        .map_err(|error| error.data.map_or(error.message, |data| data.message))
}
