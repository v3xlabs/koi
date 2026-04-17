use super::ApiTags;
use poem::Result;
use poem_openapi::{OpenApi, payload::Json};

pub struct HealthApi;

pub fn api() -> impl OpenApi {
    HealthApi
}

#[OpenApi]
impl HealthApi {
    /// List all vendor flags
    ///
    /// GET /api/health
    #[oai(path = "/health", method = "get", tag = "ApiTags::Health")]
    async fn get_health(&self) -> Result<Json<String>> {
        Ok(Json("OK".to_string()))
    }
}
