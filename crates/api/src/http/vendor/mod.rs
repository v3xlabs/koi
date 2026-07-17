use crate::{
    http::auth::Auth,
    models::vendor::flags::{VendorFlag, VendorFlagInfo},
    state::AppState,
};

use super::ApiTags;
use poem::{Result, web::Data};
use poem_openapi::{Object, OpenApi, param::Path, payload::Json};
use serde::{Deserialize, Serialize};

pub struct VendorApi;

pub fn api() -> impl OpenApi {
    VendorApi
}

#[derive(Serialize, Deserialize, Object)]
pub struct VendorsResponse {
    pub vendors: Vec<VendorFlag>,
}

#[derive(Serialize, Deserialize, Object)]
pub struct VendorFlagInfoResponse {
    pub vendors: Vec<VendorFlagInfo>,
}

#[OpenApi]
impl VendorApi {
    /// List enabled vendors
    ///
    /// GET /api/vendor
    #[oai(path = "/vendor", method = "get", tag = "ApiTags::Vendor")]
    async fn get_vendors(
        &self,
        auth: Auth,
        state: Data<&AppState>,
    ) -> Result<Json<VendorsResponse>> {
        let _auth_data = auth.unwrap()?;

        let vendors = state.vendors.all();

        Ok(Json(VendorsResponse { vendors }))
    }

    /// List all vendors
    ///
    /// GET /api/vendor/all
    #[oai(path = "/vendor/all", method = "get", tag = "ApiTags::Vendor")]
    async fn get_all_vendors(&self, auth: Auth) -> Result<Json<VendorFlagInfoResponse>> {
        let _auth_data = auth.unwrap()?;

        let vendors = VendorFlag::all();

        Ok(Json(VendorFlagInfoResponse { vendors }))
    }

    /// Set a vendor flag
    ///
    /// POST /api/vendor/:flag
    #[oai(path = "/vendor/:flag", method = "post", tag = "ApiTags::Vendor")]
    async fn enable_vendor_flag(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        flag: Path<VendorFlag>,
    ) -> Result<Json<()>> {
        let _auth_data = auth.unwrap()?;
        state.vendors.set_flag(&flag, true, &state.database).await?;

        Ok(Json(()))
    }

    /// Disable a vendor flag
    ///
    /// POST /api/vendor/:flag
    #[oai(path = "/vendor/:flag", method = "delete", tag = "ApiTags::Vendor")]
    async fn disable_vendor_flag(
        &self,
        auth: Auth,
        state: Data<&AppState>,
        flag: Path<VendorFlag>,
    ) -> Result<Json<()>> {
        let _auth_data = auth.unwrap()?;

        state
            .vendors
            .set_flag(&flag, false, &state.database)
            .await?;

        Ok(Json(()))
    }
}
