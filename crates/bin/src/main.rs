use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use koi::state::State;
#[cfg(feature = "gui")]
use koi_daemon::DAEMON_ORIGIN;
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
    Gui,
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
    Commands::Gui
}

#[cfg(not(feature = "gui"))]
fn default_command() -> Commands {
    Commands::Daemon
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    match Cli::parse().command() {
        #[cfg(feature = "gui")]
        Commands::Gui => {
            init_logging(false);
            let _server_handle = match prepare_daemon().await {
                Ok(result) => result,
                Err(error) => {
                    eprintln!("GUI startup error: {error:#}");
                    std::process::exit(1);
                }
            };

            if let Err(error) = koi_gui::run(DAEMON_ORIGIN) {
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
async fn prepare_daemon() -> anyhow::Result<Option<tokio::task::JoinHandle<()>>> {
    if daemon_is_healthy().await {
        return Ok(None);
    }

    warn!("No Koi daemon found at {DAEMON_ORIGIN}; starting one in this process");
    let state = State::new().await?;
    let handle = tokio::spawn(async move {
        if let Err(error) = koi_daemon::serve(state).await {
            tracing::error!(%error, "embedded daemon exited");
        }
    });

    wait_for_daemon(&handle).await?;
    Ok(Some(handle))
}

#[cfg(feature = "gui")]
async fn daemon_is_healthy() -> bool {
    let url = format!("{DAEMON_ORIGIN}/bootstrap");
    reqwest::get(url)
        .await
        .map(|response| response.status().is_success())
        .unwrap_or(false)
}

#[cfg(feature = "gui")]
async fn wait_for_daemon(handle: &tokio::task::JoinHandle<()>) -> anyhow::Result<()> {
    let mut last_error = None;
    for _ in 0..50 {
        if handle.is_finished() {
            anyhow::bail!("daemon task exited before becoming healthy");
        }

        if daemon_is_healthy().await {
            return Ok(());
        }

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        last_error = Some(format!(
            "daemon at {DAEMON_ORIGIN} did not become healthy yet"
        ));
    }

    anyhow::bail!(
        "{}",
        last_error.unwrap_or_else(|| format!("daemon at {DAEMON_ORIGIN} did not become healthy"))
    )
}
