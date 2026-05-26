use std::ffi::OsString;

use clap::Parser;
use dotenvy::dotenv;
use koi::{DEFAULT_API_URL, http, state::State};
use koi_cli::{Cli, Commands};
use tracing::{info, warn};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let args = std::env::args_os().collect::<Vec<_>>();
    let explicit_api = api_was_explicit(&args);

    match Cli::parse_from(args).command() {
        Commands::Gui { api } => {
            init_logging(false);
            let (api, _server_handle) = match prepare_client_api(api, explicit_api).await {
                Ok(result) => result,
                Err(error) => {
                    eprintln!("GUI startup error: {error:#}");
                    std::process::exit(1);
                }
            };

            if let Err(error) = koi_gui::run(koi_gui::GuiOptions { url: api }) {
                eprintln!("GUI error: {error:#}");
                std::process::exit(1);
            }
        }
        Commands::Serve => {
            init_logging(false);
            info!("Heya");
            let state = State::new().await.unwrap();
            http::serve(state).await;
        }
        Commands::Tui { api } => {
            init_logging(true);
            let (api, _server_handle) = match prepare_client_api(api, explicit_api).await {
                Ok(result) => result,
                Err(error) => {
                    eprintln!("TUI startup error: {error:#}");
                    std::process::exit(1);
                }
            };

            if let Err(error) = koi_tui::run(api).await {
                eprintln!("TUI error: {error:#}");
                std::process::exit(1);
            }
        }
    }
}

fn init_logging(silent: bool) {
    let builder = tracing_subscriber::fmt();
    if silent {
        builder
            .with_writer(std::io::sink)
            .with_max_level(tracing::Level::ERROR)
            .init();
    } else {
        builder.init();
    }
}

async fn prepare_client_api(
    api: String,
    explicit_api: bool,
) -> anyhow::Result<(String, Option<tokio::task::JoinHandle<()>>)> {
    if explicit_api || api.trim_end_matches('/') != DEFAULT_API_URL {
        return Ok((api, None));
    }

    if daemon_is_healthy(&api).await {
        return Ok((api, None));
    }

    warn!("No koi daemon found at {api}; starting one in this process");
    let state = State::new().await?;
    let handle = tokio::spawn(async move {
        http::serve(state).await;
    });

    wait_for_daemon(&api, &handle).await?;
    Ok((api, Some(handle)))
}

fn api_was_explicit(args: &[OsString]) -> bool {
    args.iter().any(|arg| {
        let value = arg.to_string_lossy();
        value == "--api" || value.starts_with("--api=")
    })
}

async fn daemon_is_healthy(api: &str) -> bool {
    let url = format!("{}/api/health", api.trim_end_matches('/'));
    reqwest::get(url)
        .await
        .map(|response| response.status().is_success())
        .unwrap_or(false)
}

async fn wait_for_daemon(api: &str, handle: &tokio::task::JoinHandle<()>) -> anyhow::Result<()> {
    let mut last_error = None;
    for _ in 0..50 {
        if handle.is_finished() {
            anyhow::bail!("daemon task exited before becoming healthy");
        }

        if daemon_is_healthy(api).await {
            return Ok(());
        }

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        last_error = Some(format!("daemon at {api} did not become healthy yet"));
    }

    anyhow::bail!(
        "{}",
        last_error.unwrap_or_else(|| format!("daemon at {api} did not become healthy"))
    )
}
