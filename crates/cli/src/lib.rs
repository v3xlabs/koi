use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "koi", about = "Koi privacy wallet", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Launch the desktop UI (default)
    Gui {
        /// Base URL of the koi server (without /api)
        #[arg(long, default_value = "http://localhost:7777")]
        api: String,
    },
    /// Start the HTTP server
    Serve,
    /// Launch the terminal UI (requires a running server)
    Tui {
        /// Base URL of the koi server (without /api)
        #[arg(long, default_value = "http://localhost:7777")]
        api: String,
    },
}

impl Cli {
    pub fn command(self) -> Commands {
        self.command.unwrap_or(Commands::Gui {
            api: "http://localhost:7777".to_string(),
        })
    }
}
