use poem::{Result, web::Data};
use poem_openapi::{Object, OpenApi, param::Path, payload::Json};
use serde::{Deserialize, Serialize};

use super::ApiTags;
use crate::{
    http::auth::Auth,
    models::quoter::{
        Quoter, QuoterCreate, QuoterUpdate,
        discover::{QuoterDiscovery, QuoterDiscoveryResponse},
    },
    state::AppState,
};

pub struct QuoterApi;

pub fn api() -> impl OpenApi {
    QuoterApi
}

#[derive(Serialize, Deserialize, Object)]
pub struct QuotersResponse {
    pub quoters: Vec<Quoter>,
}

#[OpenApi]
impl QuoterApi {
    /// List all quoters
    ///
    /// GET /api/quoter
    #[oai(path = "/quoter", method = "get", tag = "ApiTags::Quoter")]
    async fn get_quoters(
        &self,
        auth: Auth,
        state: Data<&AppState>,
    ) -> Result<Json<QuotersResponse>> {
        let _auth_data = auth.unwrap()?;

        let quoters = Quoter::all(&state.database).await?;

        Ok(Json(QuotersResponse { quoters }))
    }

    /// Get a quoter by id
    ///
    /// GET /api/quoter/:quoter_identity
    #[oai(
        path = "/quoter/:quoter_identity",
        method = "get",
        tag = "ApiTags::Quoter"
    )]
    async fn get_quoter_by_id(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        quoter_identity: Path<String>,
    ) -> Result<Json<Quoter>> {
        let _auth_data = auth.unwrap()?;

        let quoter = Quoter::get_by_id(&state.database, &quoter_identity).await?;

        Ok(Json(quoter))
    }

    /// Create a quoter
    ///
    /// POST /api/quoter
    #[oai(path = "/quoter", method = "post", tag = "ApiTags::Quoter")]
    async fn create_quoter(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        payload: Json<QuoterCreate>,
    ) -> Result<Json<Quoter>> {
        let _auth_data = auth.unwrap()?;

        let quoter = Quoter::insert(&state.database, payload.0).await?;

        state
            .quoters
            .build_graph(&state.database, &state.vendors)
            .await?;

        Ok(Json(quoter))
    }

    /// Update a quoter
    ///
    /// PUT /api/quoter/:quoter_identity
    #[oai(
        path = "/quoter/:quoter_identity",
        method = "put",
        tag = "ApiTags::Quoter"
    )]
    async fn update_quoter(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        quoter_identity: Path<String>,
        payload: Json<QuoterUpdate>,
    ) -> Result<Json<Quoter>> {
        let _auth_data = auth.unwrap()?;

        let quoter = Quoter::update(&state.database, &quoter_identity, payload.0).await?;

        state
            .quoters
            .build_graph(&state.database, &state.vendors)
            .await?;

        Ok(Json(quoter))
    }

    /// Discover Quoter
    ///
    /// POST /api/quoter/discover
    #[oai(path = "/quoter/discover", method = "post", tag = "ApiTags::Quoter")]
    async fn discover_quoter(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        payload: Json<QuoterDiscovery>,
    ) -> Result<Json<QuoterDiscoveryResponse>> {
        let _auth_data = auth.unwrap()?;

        Ok(Json(payload.0.discover(&state).await?))
    }
}
