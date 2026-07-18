//! Bridge API surface for the Flutter app.
//!
//! Mobile uses the same typed dispatcher as the WebSocket daemon, but calls it
//! directly without transport authentication.

use std::path::Path;

use koi::{config::Configuration, state::State};
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

pub async fn create_client(data_dir: String) -> Result<InProcessClient, String> {
    let data_dir = Path::new(&data_dir);
    if !data_dir.is_absolute() {
        return Err("application data directory must be an absolute path".to_string());
    }

    let config = Configuration::for_data_dir(data_dir);
    let state = State::new_with(config)
        .await
        .map_err(|error| error.safe_message())?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn client_uses_the_supplied_data_directory() {
        let data_dir = tempfile::tempdir().unwrap();
        let client = create_client(data_dir.path().display().to_string())
            .await
            .unwrap();

        assert_eq!(system_ping(&client).await.unwrap(), "OK");
        assert!(data_dir.path().join("koi.db").is_file());
    }

    #[tokio::test]
    async fn client_rejects_a_relative_data_directory() {
        let error = match create_client("relative/path".to_string()).await {
            Ok(_) => panic!("relative data directory was accepted"),
            Err(error) => error,
        };

        assert_eq!(error, "application data directory must be an absolute path");
    }
}
