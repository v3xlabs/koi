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
    #[oai(path = "/wallets", method = "get", tag = "ApiTags::Wallet")]
    async fn get_wallets(&self) -> Result<Json<String>> {
        Ok(Json("OK".to_string()))
    }
}
