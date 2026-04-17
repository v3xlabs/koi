use crate::http::auth::Auth;

use super::ApiTags;
use poem::Result;
use poem_openapi::{payload::Json, OpenApi};

pub struct WalletsApi;

pub fn api() -> impl OpenApi {
    (WalletsApi)
}

#[OpenApi]
impl WalletsApi {
    /// List all wallets
    ///
    /// GET /api/wallets
    #[oai(path = "/wallets", method = "get", tag = "ApiTags::Account")]
    async fn get_accounts(&self, auth: Auth) -> Result<Json<String>> {
        let auth_data = auth.unwrap()?;
        Ok(Json(auth_data.user_id))
    }
}
