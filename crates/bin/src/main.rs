use std::ffi::OsString;

use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use koi::{DEFAULT_API_URL, http, state::State};
use tracing::{info, warn};

#[derive(Parser)]
#[command(name = "koi", about = "Koi privacy wallet", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch the desktop UI (default)
    #[cfg(feature = "gui")]
    Gui {
        /// Base URL of the koi server (without /api)
        #[arg(long, default_value = "http://localhost:7777")]
        api: String,
    },
    /// Launch the terminal UI (requires a running server)
    Tui {
        /// Base URL of the koi server (without /api)
        #[arg(long, default_value = "http://localhost:7777")]
        api: String,
    },
    /// Start the daemon
    Daemon,
    /// Create the database (if needed) and apply pending migrations
    Migrate {
        /// Mark pending migrations as applied without executing SQL.
        /// Use alone to baseline a pre-existing database, or pass a version to skip through that migration inclusive.
        #[arg(long, num_args = 0..=1)]
        skip: Option<Option<i64>>,
    },
}

impl Cli {
    fn command(self) -> Commands {
        self.command.unwrap_or_else(|| default_command())
    }
}

#[cfg(feature = "gui")]
fn default_command() -> Commands {
    Commands::Gui {
        api: "http://localhost:7777".to_string(),
    }
}

#[cfg(not(feature = "gui"))]
fn default_command() -> Commands {
    Commands::Tui {
        api: "http://localhost:7777".to_string(),
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    koi::install_crypto_provider();

    let args = std::env::args_os().collect::<Vec<_>>();
    let explicit_api = api_was_explicit(&args);

    match Cli::parse_from(args).command() {
        #[cfg(feature = "gui")]
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
        Commands::Daemon => {
            init_logging(false);
            info!("Starting daemon");
            let state = State::new().await.unwrap();
            http::serve(state).await;
        }
        Commands::Migrate { skip } => {
            init_logging(false);
            if let Err(error) = State::run_migrations(skip).await {
                eprintln!("Migration error: {error:#}");
                std::process::exit(1);
            }
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
