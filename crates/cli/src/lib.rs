use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "koi", about = "Koi privacy wallet", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the HTTP server (default)
    Serve,
    /// Launch the terminal UI (requires a running server)
    Tui {
        /// Base URL of the koi server (without /api)
        #[arg(long, default_value = "http://127.0.0.1:7777")]
        api: String,
    },
}

impl Cli {
    pub fn command(self) -> Commands {
        self.command.unwrap_or(Commands::Serve)
    }
}
