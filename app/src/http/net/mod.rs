use crate::{
    error::KoiError,
    http::auth::Auth,
    models::network::{
        Network, NetworkUpdate,
        endpoint::{NetworkEndpoint, NetworkEndpointUpdate, provider::RpcStatus},
        identity::NetworkIdentity,
    },
    state::AppState,
};

use super::ApiTags;
use poem::{Result, web::Data};
use poem_openapi::{Object, OpenApi, param::Path, payload::Json};
use reqwest::StatusCode;
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

    /// Create a network
    ///
    /// POST /api/net
    #[oai(path = "/net", method = "post", tag = "ApiTags::Network")]
    async fn create_network(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        payload: Json<Network>,
    ) -> Result<Json<Network>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(Network::create(&state, payload.0).await?))
    }

    /// Get network presets
    ///
    /// GET /api/net/presets
    #[oai(path = "/net/presets", method = "get", tag = "ApiTags::Network")]
    async fn get_network_presets(&self, auth: Auth) -> Result<Json<Vec<Network>>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(Network::presets()))
    }

    /// Get a network by ID
    ///
    /// GET /api/net/:network_identity
    #[oai(path = "/net/:network_identity", method = "get", tag = "ApiTags::Network")]
    async fn get_network_by_id(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        network_identity: Path<NetworkIdentity>,
    ) -> Result<Json<Network>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(Network::get_by_id(&state, &network_identity).await?))
    }

    /// Update a network by ID
    ///
    /// PUT /api/net/:network_identity
    #[oai(path = "/net/:network_identity", method = "put", tag = "ApiTags::Network")]
    async fn update_network_by_id(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        network_identity: Path<NetworkIdentity>,
        payload: Json<NetworkUpdate>,
    ) -> Result<Json<Network>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(
            Network::update(&state, &network_identity, payload.0).await?,
        ))
    }

    /// Delete a network by ID
    ///
    /// DELETE /api/net/:network_identity
    #[oai(path = "/net/:network_identity", method = "delete", tag = "ApiTags::Network")]
    async fn delete_network_by_id(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        network_identity: Path<NetworkIdentity>,
    ) -> Result<Json<()>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(Network::delete(&state, &network_identity).await?))
    }

    /// Get network endpoints
    ///
    /// GET /api/net/:network_identity/endpoints
    #[oai(
        path = "/net/:network_identity/endpoints",
        method = "get",
        tag = "ApiTags::Network"
    )]
    async fn get_network_endpoints(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        network_identity: Path<NetworkIdentity>,
    ) -> Result<Json<Vec<NetworkEndpoint>>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(
            NetworkEndpoint::get_by_network_id(&state, &network_identity.0).await?,
        ))
    }

    /// Get the next network endpoint ID
    ///
    /// GET /api/net/:network_identity/endpoints/next-id
    #[oai(
        path = "/net/:network_identity/endpoints/next-id",
        method = "get",
        tag = "ApiTags::Network"
    )]
    async fn get_network_endpoint_next_id(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        network_identity: Path<NetworkIdentity>,
    ) -> Result<Json<i32>> {
        let _auth_data = auth.unwrap()?;
        let _network_identity = network_identity.0;
        Ok(Json(NetworkEndpoint::get_next_id(&state).await?))
    }

    /// Create a network endpoint
    ///
    /// POST /api/net/:network_identity/endpoints
    #[oai(
        path = "/net/:network_identity/endpoints",
        method = "post",
        tag = "ApiTags::Network"
    )]
    async fn create_network_endpoint(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        network_identity: Path<NetworkIdentity>,
        payload: Json<NetworkEndpoint>,
    ) -> Result<Json<NetworkEndpoint>> {
        let _auth_data = auth.unwrap()?;

        if payload.0.network_identity != network_identity.0 {
            return Err(poem::Error::from_status(StatusCode::BAD_REQUEST));
        }

        Ok(Json(NetworkEndpoint::create(&state, payload.0).await?))
    }

    /// Get a network endpoint by ID
    ///
    /// GET /api/net/:network_identity/endpoints/:endpoint_identity
    #[oai(
        path = "/net/:network_identity/endpoints/:endpoint_identity",
        method = "get",
        tag = "ApiTags::Network"
    )]
    async fn get_network_endpoint_by_id(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        network_identity: Path<NetworkIdentity>,
        endpoint_identity: Path<i32>,
    ) -> Result<Json<NetworkEndpoint>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(
            NetworkEndpoint::get_by_id(&state.database, &network_identity.0, &endpoint_identity.0).await?,
        ))
    }

    /// Update a network endpoint by ID
    ///
    /// PUT /api/net/:network_identity/endpoints/:endpoint_identity
    #[oai(
        path = "/net/:network_identity/endpoints/:endpoint_identity",
        method = "put",
        tag = "ApiTags::Network"
    )]
    async fn update_network_endpoint_by_id(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        network_identity: Path<NetworkIdentity>,
        endpoint_identity: Path<i32>,
        payload: Json<NetworkEndpointUpdate>,
    ) -> Result<Json<NetworkEndpoint>> {
        let _auth_data = auth.unwrap()?;

        let updated_endpoint =
            NetworkEndpoint::update(&state, &network_identity, &endpoint_identity, payload.0).await?;

        // Notify the running rpc
        state
            .networks
            .get_pool(&network_identity)
            .get_rpc(&endpoint_identity, &state.database)
            .await
            .update(&updated_endpoint)
            .await
            .map_err(KoiError::from)?;

        Ok(Json(updated_endpoint))
    }

    /// Delete a network endpoint by ID
    ///
    /// DELETE /api/net/:network_identity/endpoints/:endpoint_identity
    #[oai(
        path = "/net/:network_identity/endpoints/:endpoint_identity",
        method = "delete",
        tag = "ApiTags::Network"
    )]
    async fn delete_network_endpoint_by_id(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        network_identity: Path<NetworkIdentity>,
        endpoint_identity: Path<i32>,
    ) -> Result<Json<()>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(
            NetworkEndpoint::delete(&state, &network_identity, &endpoint_identity).await?,
        ))
    }

    /// Get a network endpoint status
    ///
    /// GET /api/net/:network_identity/endpoints/:endpoint_identity/status
    #[oai(
        path = "/net/:network_identity/endpoints/:endpoint_identity/status",
        method = "get",
        tag = "ApiTags::Network"
    )]
    async fn get_network_endpoint_status(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        network_identity: Path<NetworkIdentity>,
        endpoint_identity: Path<i32>,
    ) -> Result<Json<RpcStatus>> {
        let _auth_data = auth.unwrap()?;

        let pool = state.networks.get_pool(&network_identity);

        let rpc = pool.get_rpc(&endpoint_identity, &state.database).await;

        Ok(Json(rpc.get_status().await))
    }
}
