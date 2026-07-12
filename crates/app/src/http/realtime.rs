use futures::{SinkExt, StreamExt};
use poem::{
    IntoResponse, handler,
    web::{
        Data,
        websocket::{Message, WebSocket},
    },
};
use serde_json::json;
use tokio::sync::broadcast;

use crate::{models::event::AppEvent, state::AppState};

#[handler]
pub async fn realtime(ws: WebSocket, state: Data<&AppState>) -> impl IntoResponse {
    let receiver = state.events.subscribe();

    ws.on_upgrade(move |socket| handle_socket(socket, receiver))
}

async fn handle_socket(
    socket: poem::web::websocket::WebSocketStream,
    mut receiver: broadcast::Receiver<AppEvent>,
) {
    let (mut sink, mut stream) = socket.split();

    loop {
        tokio::select! {
            event = receiver.recv() => {
                let message = match event {
                    Ok(event) => event_message(&event),
                    Err(broadcast::error::RecvError::Lagged(_)) => event_message(&AppEvent::InvalidateAll),
                    Err(broadcast::error::RecvError::Closed) => break,
                };

                if sink.send(message).await.is_err() {
                    break;
                }
            }
            message = stream.next() => {
                let Some(Ok(message)) = message else {
                    break;
                };

                match message {
                    Message::Ping(payload) => {
                        if sink.send(Message::Pong(payload)).await.is_err() {
                            break;
                        }
                    }
                    Message::Close(_) => break,
                    Message::Text(_) | Message::Binary(_) | Message::Pong(_) => {}
                }
            }
        }
    }
}

fn event_message(event: &AppEvent) -> Message {
    let payload = match event {
        AppEvent::Invalidate { route } => json!({ "type": "invalidate", "route": route }),
        AppEvent::InvalidateAll => json!({ "type": "invalidate_all" }),
    };

    Message::text(payload.to_string())
}
