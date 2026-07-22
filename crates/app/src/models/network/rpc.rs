use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    Network, NetworkUpdate as NetworkUpdateInput,
    endpoint::{
        NetworkEndpoint, NetworkEndpointCreate, NetworkEndpointUpdate, provider::RpcStatus,
    },
    identity::NetworkIdentity,
    metadata::NetworkMetadataDiscovery,
    pool::RpcPoolStats,
};
use crate::{
    error::KoiError,
    rpc::{EmptyParams, RpcHandler},
    rpc_method,
    state::AppState,
};

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct NetworkParams {
    pub network_identity: NetworkIdentity,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct NetworkCreateParams {
    pub input: Network,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct NetworkUpdateParams {
    pub network_identity: NetworkIdentity,
    pub input: NetworkUpdateInput,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct EndpointParams {
    pub network_identity: NetworkIdentity,
    pub endpoint_identity: i32,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct EndpointCreateParams {
    pub network_identity: NetworkIdentity,
    pub input: NetworkEndpointCreate,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct EndpointUpdateParams {
    pub network_identity: NetworkIdentity,
    pub endpoint_identity: i32,
    pub input: NetworkEndpointUpdate,
}

rpc_method!(NetworkList, "network.list", EmptyParams => Vec<Network>);
rpc_method!(NetworkGet, "network.get", NetworkParams => Network);
rpc_method!(NetworkCreate, "network.create", NetworkCreateParams => Network);
rpc_method!(NetworkUpdate, "network.update", NetworkUpdateParams => Network);
rpc_method!(NetworkDelete, "network.delete", NetworkParams => ());
rpc_method!(NetworkPresets, "network.presets", EmptyParams => Vec<Network>);
rpc_method!(NetworkDiscover, "network.discover", NetworkParams => NetworkMetadataDiscovery);
rpc_method!(NetworkStats, "network.stats", NetworkParams => RpcPoolStats);
rpc_method!(EndpointList, "network.endpoint.list", NetworkParams => Vec<NetworkEndpoint>);
rpc_method!(EndpointGet, "network.endpoint.get", EndpointParams => NetworkEndpoint);
rpc_method!(EndpointCreate, "network.endpoint.create", EndpointCreateParams => NetworkEndpoint);
rpc_method!(EndpointUpdate, "network.endpoint.update", EndpointUpdateParams => NetworkEndpoint);
rpc_method!(EndpointDelete, "network.endpoint.delete", EndpointParams => ());
rpc_method!(EndpointStatus, "network.endpoint.status", EndpointParams => RpcStatus);

impl RpcHandler for NetworkList {
    async fn handle(state: &AppState, _params: EmptyParams) -> Result<Vec<Network>, KoiError> {
        Network::all(&state.database).await
    }
}

impl RpcHandler for NetworkGet {
    async fn handle(state: &AppState, params: NetworkParams) -> Result<Network, KoiError> {
        Network::get_by_id(&state.database, &params.network_identity).await
    }
}

impl RpcHandler for NetworkCreate {
    async fn handle(state: &AppState, params: NetworkCreateParams) -> Result<Network, KoiError> {
        Network::create(&state.database, params.input).await
    }
}

impl RpcHandler for NetworkUpdate {
    async fn handle(state: &AppState, params: NetworkUpdateParams) -> Result<Network, KoiError> {
        Network::update(&state.database, &params.network_identity, params.input).await
    }
}

impl RpcHandler for NetworkDelete {
    async fn handle(state: &AppState, params: NetworkParams) -> Result<(), KoiError> {
        Network::delete(&state.database, &params.network_identity).await
    }
}

impl RpcHandler for NetworkPresets {
    async fn handle(_state: &AppState, _params: EmptyParams) -> Result<Vec<Network>, KoiError> {
        Ok(Network::presets())
    }
}

impl RpcHandler for NetworkDiscover {
    async fn handle(
        state: &AppState,
        params: NetworkParams,
    ) -> Result<NetworkMetadataDiscovery, KoiError> {
        Network::fetch_metadata(state, &params.network_identity).await
    }
}

impl RpcHandler for NetworkStats {
    async fn handle(state: &AppState, params: NetworkParams) -> Result<RpcPoolStats, KoiError> {
        Ok(state.networks.get_pool(&params.network_identity).snapshot())
    }
}

impl RpcHandler for EndpointList {
    async fn handle(
        state: &AppState,
        params: NetworkParams,
    ) -> Result<Vec<NetworkEndpoint>, KoiError> {
        NetworkEndpoint::get_by_network_id(&state.database, &params.network_identity).await
    }
}

impl RpcHandler for EndpointGet {
    async fn handle(state: &AppState, params: EndpointParams) -> Result<NetworkEndpoint, KoiError> {
        NetworkEndpoint::get_by_id(
            &state.database,
            &params.network_identity,
            &params.endpoint_identity,
        )
        .await
    }
}

impl RpcHandler for EndpointCreate {
    async fn handle(
        state: &AppState,
        params: EndpointCreateParams,
    ) -> Result<NetworkEndpoint, KoiError> {
        NetworkEndpoint::create(&state.database, params.network_identity, params.input).await
    }
}

impl RpcHandler for EndpointUpdate {
    async fn handle(
        state: &AppState,
        params: EndpointUpdateParams,
    ) -> Result<NetworkEndpoint, KoiError> {
        let endpoint = NetworkEndpoint::update(
            &state.database,
            &params.network_identity,
            &params.endpoint_identity,
            params.input,
        )
        .await?;
        state
            .networks
            .get_pool(&params.network_identity)
            .get_rpc(&params.endpoint_identity, state)
            .await
            .update(&endpoint)
            .await
            .map_err(KoiError::from)?;
        Ok(endpoint)
    }
}

impl RpcHandler for EndpointDelete {
    async fn handle(state: &AppState, params: EndpointParams) -> Result<(), KoiError> {
        NetworkEndpoint::delete(
            &state.database,
            &params.network_identity,
            &params.endpoint_identity,
        )
        .await?;
        state
            .networks
            .get_pool(&params.network_identity)
            .remove_endpoint(&params.endpoint_identity);
        Ok(())
    }
}

impl RpcHandler for EndpointStatus {
    async fn handle(state: &AppState, params: EndpointParams) -> Result<RpcStatus, KoiError> {
        Ok(state
            .networks
            .get_pool(&params.network_identity)
            .get_rpc(&params.endpoint_identity, state)
            .await
            .get_status()
            .await)
    }
}
