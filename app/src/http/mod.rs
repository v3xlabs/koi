use poem::endpoint::EmbeddedFilesEndpoint;
use poem::{EndpointExt, Route, Server, handler, listener::TcpListener, web::Html};
use poem_openapi::Tags;
use poem_openapi::{OpenApi, OpenApiService};
use rust_embed::RustEmbed;
use tracing::info;

use crate::state::AppState;

mod account;
mod auth;
mod health;

#[derive(Tags)]
pub enum ApiTags {
    /// Network endpoints
    Network,
    /// Vendor endpoints
    Vendor,
    /// Account endpoints
    Account,
    /// Token endpoints
    Token,
    /// Quoter endpoints
    Quoter,
    /// Settings endpoints
    Settings,
    /// Background task endpoints
    Task,
    /// Health endpoints
    Health,
}

fn get_api() -> impl OpenApi {
    (health::api(), account::api())
}

#[derive(RustEmbed)]
#[folder = "../ui/dist"]
struct WebAssets;

#[handler]
async fn get_openapi_docs() -> Html<&'static str> {
    Html(include_str!("docs.html"))
}

pub async fn serve(state: AppState) {
    info!("Serving HTTP server");

    let addr = "127.0.0.1:7777";
    let listener = TcpListener::bind(addr);
    let listener_url = format!("http://{}", addr);

    let title = "Koi";
    let description = "API";
    let server_url = format!("{}/api/", listener_url);
    let cargo_version = env!("CARGO_PKG_VERSION");

    let service = OpenApiService::new(get_api(), title, cargo_version)
        .description(description)
        .server(server_url);

    let frontend = EmbeddedFilesEndpoint::<WebAssets>::new().index_file("index.html");

    let app = Route::new()
        .nest("openapi.json", service.spec_endpoint())
        .nest("/api", service)
        .at("/docs", get_openapi_docs)
        .at("/*", frontend)
        .data(state);

    info!("You can visit the interface at {}", listener_url);

    Server::new(listener).run(app).await.unwrap()
}
