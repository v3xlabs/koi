use dotenvy::dotenv;
use tracing::info;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    info!("Heya");
}
