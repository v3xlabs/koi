//! Loopback HTTP and WebSocket transport for the Koi RPC dispatcher.

use std::sync::Arc;

use futures::{SinkExt, StreamExt, future::join_all};
use koi::state::AppState;
use koi_api::rpc::{Dispatcher, MAX_IN_FLIGHT_CALLS, MAX_MESSAGE_BYTES};
use poem::{
    EndpointExt, IntoResponse, Request, Response, Route, Server, handler,
    http::{StatusCode, header},
    listener::TcpListener,
    web::{
        Data,
        websocket::{Message, WebSocket, WebSocketStream},
    },
};
use rust_embed::RustEmbed;
use tokio::{
    sync::{Semaphore, mpsc},
    task::JoinSet,
};
use tracing::info;

pub const LISTEN_ADDRESS: &str = "localhost:7777";
pub const DAEMON_ORIGIN: &str = "http://localhost:7777";

#[derive(Clone)]
struct DaemonState {
    dispatcher: Dispatcher,
}

#[derive(RustEmbed)]
#[folder = "../../interfaces/web/dist"]
struct WebAssets;

pub async fn serve(state: AppState) -> std::io::Result<()> {
    let daemon = DaemonState {
        dispatcher: Dispatcher::new(state),
    };
    let frontend =
        poem::endpoint::EmbeddedFilesEndpoint::<WebAssets>::new().index_file("index.html");
    let app = Route::new()
        .at("/rpc", poem::get(rpc))
        .at("/*", frontend)
        .data(daemon);

    info!(origin = DAEMON_ORIGIN, "serving Koi daemon");
    Server::new(TcpListener::bind(LISTEN_ADDRESS))
        .run(app)
        .await
}

#[handler]
async fn rpc(req: &Request, ws: WebSocket, Data(state): Data<&DaemonState>) -> Response {
    if !valid_origin(req) {
        return StatusCode::FORBIDDEN.into_response();
    }

    let dispatcher = state.dispatcher.clone();
    ws.on_upgrade(move |socket| serve_socket(socket, dispatcher))
        .into_response()
}

async fn serve_socket(socket: WebSocketStream, dispatcher: Dispatcher) {
    let (mut sink, mut source) = socket.split();
    let (outgoing, mut responses) = mpsc::unbounded_channel::<Message>();
    let writer = tokio::spawn(async move {
        while let Some(message) = responses.recv().await {
            if sink.send(message).await.is_err() {
                break;
            }
        }
    });
    let permits = Arc::new(Semaphore::new(MAX_IN_FLIGHT_CALLS));
    let mut calls = JoinSet::new();

    while let Some(message) = source.next().await {
        let Ok(message) = message else {
            break;
        };
        match message {
            Message::Text(text) if text.len() <= MAX_MESSAGE_BYTES => {
                let dispatcher = dispatcher.clone();
                let outgoing = outgoing.clone();
                let permits = permits.clone();
                calls.spawn(async move {
                    if let Some(response) = process_message(&dispatcher, &permits, &text).await {
                        let _ = outgoing.send(Message::text(response));
                    }
                });
            }
            Message::Text(_) => {
                let _ = outgoing.send(Message::close_with(1009, "message exceeds 8 MiB"));
                break;
            }
            Message::Ping(payload) => {
                let _ = outgoing.send(Message::Pong(payload));
            }
            Message::Close(_) => break,
            Message::Binary(_) => {
                let _ = outgoing.send(Message::close_with(1003, "JSON-RPC requires text messages"));
                break;
            }
            Message::Pong(_) => {}
        }
    }

    calls.abort_all();
    while calls.join_next().await.is_some() {}
    drop(outgoing);
    let _ = writer.await;
}

async fn process_message(
    dispatcher: &Dispatcher,
    permits: &Arc<Semaphore>,
    text: &str,
) -> Option<String> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
        return dispatcher.process_message(text).await;
    };

    let response = match value {
        serde_json::Value::Array(values)
            if !values.is_empty() && values.len() <= koi_api::rpc::MAX_BATCH_ENTRIES =>
        {
            let responses = join_all(values.into_iter().map(|value| {
                let dispatcher = dispatcher.clone();
                let permits = permits.clone();
                async move {
                    let Ok(_permit) = permits.acquire_owned().await else {
                        return None;
                    };
                    dispatcher.process_request(value).await
                }
            }))
            .await
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

            (!responses.is_empty()).then(|| serde_json::Value::Array(responses))
        }
        value @ serde_json::Value::Array(_) => {
            return dispatcher.process_message(&value.to_string()).await;
        }
        value => {
            let Ok(_permit) = permits.clone().acquire_owned().await else {
                return None;
            };
            dispatcher.process_request(value).await
        }
    };

    response.and_then(|value| serde_json::to_string(&value).ok())
}

fn valid_origin(request: &Request) -> bool {
    let headers = request.headers();
    let Some(origin) = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };
    let Some(host) = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };

    origin == format!("http://{host}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn websocket_requires_matching_request_host() {
        let request = Request::builder()
            .header(header::ORIGIN, DAEMON_ORIGIN)
            .header(header::HOST, LISTEN_ADDRESS)
            .finish();

        assert!(valid_origin(&request));
    }

    #[test]
    fn websocket_accepts_alternate_loopback_origin() {
        let request = Request::builder()
            .header(header::ORIGIN, "http://127.0.0.1:7777")
            .header(header::HOST, "127.0.0.1:7777")
            .finish();

        assert!(valid_origin(&request));
    }

    #[test]
    fn websocket_rejects_missing_and_disallowed_origins() {
        let missing = Request::builder().finish();
        let disallowed = Request::builder()
            .header(header::ORIGIN, "https://example.com")
            .header(header::HOST, LISTEN_ADDRESS)
            .finish();
        let mismatched = Request::builder()
            .header(header::ORIGIN, "http://example.com:7777")
            .header(header::HOST, LISTEN_ADDRESS)
            .finish();

        assert!(!valid_origin(&missing));
        assert!(!valid_origin(&disallowed));
        assert!(!valid_origin(&mismatched));
    }
}
