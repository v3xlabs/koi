use crate::{
    http::auth::Auth,
    models::{
        account::identity::AccountIdentity, connection::ActivateAppConnection,
        network::identity::NetworkIdentity,
    },
    state::AppState,
};

use super::ApiTags;
use poem::{Result, web::Data};
use poem_openapi::{Object, OpenApi, param::Path, payload::Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct ConnectionApi;

pub fn api() -> impl OpenApi {
    ConnectionApi
}

#[derive(Serialize, Deserialize, Object)]
pub struct ConnectionsResponse {
    pub connections: Vec<ActivateAppConnection>,
}

#[derive(Serialize, Deserialize, Object)]
pub struct ConnectConnectionRequest {
    pub url: String,
    pub account_identity: AccountIdentity,
    pub network_identity: NetworkIdentity,
}

#[OpenApi]
impl ConnectionApi {
    /// List all OpenLV connections
    ///
    /// GET /api/connections
    #[oai(path = "/connections", method = "get", tag = "ApiTags::Connection")]
    async fn get_connections(
        &self,
        auth: Auth,
        state: Data<&AppState>,
    ) -> Result<Json<ConnectionsResponse>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(ConnectionsResponse {
            connections: state.connections.all().await,
        }))
    }

    /// Create an OpenLV connection
    ///
    /// POST /api/connections
    #[oai(path = "/connections", method = "post", tag = "ApiTags::Connection")]
    async fn create_connection(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        payload: Json<ConnectConnectionRequest>,
    ) -> Result<Json<ActivateAppConnection>> {
        let _auth_data = auth.unwrap()?;
        let request = payload.0;

        Ok(Json(
            state
                .connections
                .connect(
                    &state,
                    request.url,
                    request.account_identity,
                    request.network_identity,
                )
                .await?,
        ))
    }

    /// Disconnect an OpenLV connection
    ///
    /// POST /api/connections/:connection_id/disconnect
    #[oai(
        path = "/connections/:connection_id/disconnect",
        method = "post",
        tag = "ApiTags::Connection"
    )]
    async fn disconnect_connection(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        connection_id: Path<Uuid>,
    ) -> Result<Json<ActivateAppConnection>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(state.connections.disconnect(connection_id.0).await?))
    }

    /// Remove an OpenLV connection
    ///
    /// DELETE /api/connections/:connection_id
    #[oai(
        path = "/connections/:connection_id",
        method = "delete",
        tag = "ApiTags::Connection"
    )]
    async fn remove_connection(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        connection_id: Path<Uuid>,
    ) -> Result<Json<()>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(state.connections.remove(connection_id.0).await?))
    }
}
