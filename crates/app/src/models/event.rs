use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AppEvent {
    Invalidate { route: String },
    InvalidateAll,
}

#[derive(Clone)]
pub struct AppEventBus {
    events: broadcast::Sender<AppEvent>,
}

impl AppEventBus {
    pub fn new() -> Self {
        let (events, _) = broadcast::channel(1024);

        Self { events }
    }

    pub fn invalidate_route(&self, route: impl Into<String>) {
        let _ = self.events.send(AppEvent::Invalidate {
            route: route.into(),
        });
    }

    pub fn invalidate_all(&self) {
        let _ = self.events.send(AppEvent::InvalidateAll);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.events.subscribe()
    }
}

impl Default for AppEventBus {
    fn default() -> Self {
        Self::new()
    }
}
