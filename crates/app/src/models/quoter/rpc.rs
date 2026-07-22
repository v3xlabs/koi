use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    Quoter, QuoterCreate as QuoterCreateInput, QuoterUpdate as QuoterUpdateInput,
    discover::{QuoterDiscovery, QuoterDiscoveryResponse},
};
use crate::{
    error::KoiError,
    rpc::{EmptyParams, RpcHandler},
    rpc_method,
    state::AppState,
};

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct QuoterParams {
    pub quoter_identity: String,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct QuoterCreateParams {
    pub input: QuoterCreateInput,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct QuoterUpdateParams {
    pub quoter_identity: String,
    pub input: QuoterUpdateInput,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct QuoterDiscoverParams {
    pub input: QuoterDiscovery,
}

rpc_method!(QuoterList, "quoter.list", EmptyParams => Vec<Quoter>);
rpc_method!(QuoterGet, "quoter.get", QuoterParams => Quoter);
rpc_method!(QuoterCreate, "quoter.create", QuoterCreateParams => Quoter);
rpc_method!(QuoterUpdate, "quoter.update", QuoterUpdateParams => Quoter);
rpc_method!(QuoterDiscover, "quoter.discover", QuoterDiscoverParams => QuoterDiscoveryResponse);

impl RpcHandler for QuoterList {
    async fn handle(state: &AppState, _params: EmptyParams) -> Result<Vec<Quoter>, KoiError> {
        Quoter::all(&state.database).await
    }
}

impl RpcHandler for QuoterGet {
    async fn handle(state: &AppState, params: QuoterParams) -> Result<Quoter, KoiError> {
        Quoter::get_by_id(&state.database, &params.quoter_identity).await
    }
}

impl RpcHandler for QuoterCreate {
    async fn handle(state: &AppState, params: QuoterCreateParams) -> Result<Quoter, KoiError> {
        let quoter = Quoter::insert(&state.database, params.input).await?;
        state.quoters.build_graph(&state.database).await?;
        Ok(quoter)
    }
}

impl RpcHandler for QuoterUpdate {
    async fn handle(state: &AppState, params: QuoterUpdateParams) -> Result<Quoter, KoiError> {
        let quoter = Quoter::update(&state.database, &params.quoter_identity, params.input).await?;
        state.quoters.build_graph(&state.database).await?;
        Ok(quoter)
    }
}

impl RpcHandler for QuoterDiscover {
    async fn handle(
        state: &AppState,
        params: QuoterDiscoverParams,
    ) -> Result<QuoterDiscoveryResponse, KoiError> {
        params.input.discover(state).await
    }
}
