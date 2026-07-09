use std::{collections::HashMap, sync::Arc};

use openlv::{Session, SessionState, wallet};
use poem_openapi::{Enum, Object};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::models::{
    account::identity::AccountIdentity, network::identity::NetworkIdentity,
    wallet_request::WalletRequestManager,
};
use crate::{error::KoiError, models::event::AppEventBus};

#[derive(Clone, Debug, Serialize, Deserialize, Object)]
pub struct FrontendConnection {
    pub connection_id: Uuid,
    pub status: ConnectionStatus,
    pub account_identity: AccountIdentity,
    pub network_identity: NetworkIdentity,
}

struct BackendConnection {
    connection_id: Uuid,
    session: Arc<Session>,
    account_identity: AccountIdentity,
    network_identity: NetworkIdentity,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Enum)]
#[oai(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ConnectionStatus {
    Created,
    Signaling,
    Ready,
    Linking,
    Connected,
    Disconnected,
}

impl BackendConnection {
    fn to_response(&self) -> FrontendConnection {
        FrontendConnection {
            connection_id: self.connection_id,
            status: self.session.state().status.into(),
            account_identity: self.account_identity.clone(),
            network_identity: self.network_identity.clone(),
        }
    }
}

pub struct ConnectionManager {
    connections: RwLock<HashMap<Uuid, BackendConnection>>,
    events: AppEventBus,
    requests: WalletRequestManager,
}

impl ConnectionManager {
    pub fn new(events: AppEventBus, requests: WalletRequestManager) -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
            events,
            requests,
        }
    }

    pub async fn all(&self) -> Vec<FrontendConnection> {
        self.connections
            .read()
            .await
            .values()
            .map(BackendConnection::to_response)
            .collect()
    }

    pub async fn connect(
        &self,
        url: String,
        account_identity: AccountIdentity,
        network_identity: NetworkIdentity,
    ) -> Result<FrontendConnection, KoiError> {
        let connection_id = Uuid::new_v4();
        let requests = self.requests.clone();
        let request_account = account_identity.clone();
        let request_network = network_identity.clone();
        let session = wallet(&url)
            .on_request(move |message: Value| {
                let requests = requests.clone();
                let account_identity = request_account.clone();
                let network_identity = request_network.clone();

                async move {
                    requests
                        .handle_openlv_request(
                            connection_id,
                            account_identity,
                            network_identity,
                            message,
                        )
                        .await
                }
            })
            .await?;
        session.connect().await?;

        let connection = BackendConnection {
            connection_id,
            account_identity,
            network_identity,
            session: Arc::new(session),
        };
        let response = connection.to_response();
        let session = Arc::clone(&connection.session);

        self.connections
            .write()
            .await
            .insert(connection.connection_id, connection);
        self.spawn_watchers(session);
        self.notify_changed();

        Ok(response)
    }

    pub async fn disconnect(&self, connection_id: Uuid) -> Result<FrontendConnection, KoiError> {
        self.session(connection_id).await?.close().await?;
        self.requests
            .reject_connection(connection_id, "Connection disconnected")
            .await;
        self.notify_changed();

        self.connection(connection_id).await
    }

    pub async fn remove(&self, connection_id: Uuid) -> Result<(), KoiError> {
        let entry = self
            .connections
            .write()
            .await
            .remove(&connection_id)
            .ok_or_else(|| connection_not_found(connection_id))?;

        if entry.session.state().status != SessionState::Disconnected {
            entry.session.close().await?;
        }
        self.requests
            .reject_connection(connection_id, "Connection removed")
            .await;
        self.notify_changed();

        Ok(())
    }

    async fn connection(&self, connection_id: Uuid) -> Result<FrontendConnection, KoiError> {
        self.connections
            .read()
            .await
            .get(&connection_id)
            .map(BackendConnection::to_response)
            .ok_or_else(|| connection_not_found(connection_id))
    }

    async fn session(&self, connection_id: Uuid) -> Result<Arc<Session>, KoiError> {
        self.connections
            .read()
            .await
            .get(&connection_id)
            .map(|entry| Arc::clone(&entry.session))
            .ok_or_else(|| connection_not_found(connection_id))
    }

    fn notify_changed(&self) {
        self.events.invalidate_route("/connections");
    }

    fn spawn_watchers(&self, session: Arc<Session>) {
        let mut state_rx = session.subscribe_state();
        let events = self.events.clone();

        // Spawn a task to watch for state changes and notify the event bus
        tokio::spawn(async move {
            while state_rx.recv().await.is_ok() {
                events.invalidate_route("/connections");
            }
        });

        // Spawn a task to update public connection status when the session is ready
        tokio::spawn(async move {
            if let Err(error) = session.wait_for_link().await {
                tracing::debug!("openlv link waiter ended: {error}");
            }
        });
    }
}

fn connection_not_found(connection_id: Uuid) -> KoiError {
    KoiError::Internal(format!("connection not found: {connection_id}"))
}

impl From<SessionState> for ConnectionStatus {
    fn from(value: SessionState) -> Self {
        match value {
            SessionState::Created => Self::Created,
            SessionState::Signaling => Self::Signaling,
            SessionState::Ready => Self::Ready,
            SessionState::Linking => Self::Linking,
            SessionState::Connected => Self::Connected,
            SessionState::Disconnected => Self::Disconnected,
        }
    }
}
