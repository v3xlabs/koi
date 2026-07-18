#[cfg(feature = "gui")]
use std::ffi::OsString;

use clap::{Parser, Subcommand};
use dotenvy::dotenv;
#[cfg(feature = "gui")]
use koi::DEFAULT_DAEMON_URL;
use koi::state::State;
use tracing::info;
#[cfg(feature = "gui")]
use tracing::warn;

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
        /// Base URL of the local Koi daemon.
        #[arg(long, default_value = DEFAULT_DAEMON_URL)]
        daemon_url: String,
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
        self.command.unwrap_or_else(default_command)
    }
}

#[cfg(feature = "gui")]
fn default_command() -> Commands {
    Commands::Gui {
        daemon_url: DEFAULT_DAEMON_URL.to_string(),
    }
}

#[cfg(not(feature = "gui"))]
fn default_command() -> Commands {
    Commands::Daemon
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let args = std::env::args_os().collect::<Vec<_>>();
    #[cfg(feature = "gui")]
    let explicit_daemon_url = daemon_url_was_explicit(&args);

    match Cli::parse_from(args).command() {
        #[cfg(feature = "gui")]
        Commands::Gui { daemon_url } => {
            init_logging(false);
            let (daemon_url, _server_handle) =
                match prepare_daemon(daemon_url, explicit_daemon_url).await {
                    Ok(result) => result,
                    Err(error) => {
                        eprintln!("GUI startup error: {error:#}");
                        std::process::exit(1);
                    }
                };

            if let Err(error) = koi_gui::run(koi_gui::GuiOptions { url: daemon_url }) {
                eprintln!("GUI error: {error:#}");
                std::process::exit(1);
            }
        }
        Commands::Daemon => {
            init_logging(false);
            info!("Starting daemon");
            let state = match State::new().await {
                Ok(state) => state,
                Err(error) => {
                    eprintln!("Daemon startup error: {}", error.safe_message());
                    std::process::exit(1);
                }
            };
            if let Err(error) = koi_daemon::serve(state).await {
                eprintln!("Daemon error: {error}");
                std::process::exit(1);
            }
        }
        Commands::Migrate { skip } => {
            init_logging(false);
            if let Err(error) = State::run_migrations(skip).await {
                eprintln!("Migration error: {error:#}");
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

#[cfg(feature = "gui")]
async fn prepare_daemon(
    daemon_url: String,
    explicit_daemon_url: bool,
) -> anyhow::Result<(String, Option<tokio::task::JoinHandle<()>>)> {
    if explicit_daemon_url || daemon_url.trim_end_matches('/') != DEFAULT_DAEMON_URL {
        return Ok((daemon_url, None));
    }

    if daemon_is_healthy(&daemon_url).await {
        return Ok((daemon_url, None));
    }

    warn!("No Koi daemon found at {daemon_url}; starting one in this process");
    let state = State::new().await?;
    let handle = tokio::spawn(async move {
        if let Err(error) = koi_daemon::serve(state).await {
            tracing::error!(%error, "embedded daemon exited");
        }
    });

    wait_for_daemon(&daemon_url, &handle).await?;
    Ok((daemon_url, Some(handle)))
}

#[cfg(feature = "gui")]
fn daemon_url_was_explicit(args: &[OsString]) -> bool {
    args.iter().any(|arg| {
        let value = arg.to_string_lossy();
        value == "--daemon-url" || value.starts_with("--daemon-url=")
    })
}

#[cfg(feature = "gui")]
async fn daemon_is_healthy(daemon_url: &str) -> bool {
    let url = format!("{}/bootstrap", daemon_url.trim_end_matches('/'));
    reqwest::get(url)
        .await
        .map(|response| response.status().is_success())
        .unwrap_or(false)
}

#[cfg(feature = "gui")]
async fn wait_for_daemon(
    daemon_url: &str,
    handle: &tokio::task::JoinHandle<()>,
) -> anyhow::Result<()> {
    let mut last_error = None;
    for _ in 0..50 {
        if handle.is_finished() {
            anyhow::bail!("daemon task exited before becoming healthy");
        }

        if daemon_is_healthy(daemon_url).await {
            return Ok(());
        }

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        last_error = Some(format!("daemon at {daemon_url} did not become healthy yet"));
    }

    anyhow::bail!(
        "{}",
        last_error.unwrap_or_else(|| format!("daemon at {daemon_url} did not become healthy"))
    )
}
