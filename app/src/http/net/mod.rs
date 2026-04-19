use crate::{http::auth::Auth, models::network::Network, state::AppState};

use super::ApiTags;
use poem::{Result, web::Data};
use poem_openapi::{Object, OpenApi, param::Path, payload::Json};
use serde::{Deserialize, Serialize};

pub struct NetworkApi;

pub fn api() -> impl OpenApi {
    NetworkApi
}

#[derive(Serialize, Deserialize, Object)]
pub struct NetworksResponse {
    pub networks: Vec<Network>,
}

#[OpenApi]
impl NetworkApi {
    /// List all networks
    ///
    /// GET /api/net
    #[oai(path = "/net", method = "get", tag = "ApiTags::Network")]
    async fn get_networks(
        &self,
        auth: Auth,
        state: Data<&AppState>,
    ) -> Result<Json<NetworksResponse>> {
        let _auth_data = auth.unwrap()?;

        let networks = Network::all(&state).await?;

        Ok(Json(NetworksResponse { networks }))
    }

    /// Get a network by ID
    ///
    /// GET /api/net/:network_id
    #[oai(path = "/net/:network_id", method = "get", tag = "ApiTags::Network")]
    async fn get_network_by_id(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        network_id: Path<i32>,
    ) -> Result<Json<Network>> {
        let _auth_data = auth.unwrap()?;

        let network = Network::get_by_id(&state, network_id.0).await?;

        Ok(Json(network))
    }
}
