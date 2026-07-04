use poem::{Result, http::header, web::Data};
use poem_openapi::{
    OpenApi,
    param::Query,
    payload::{Binary, Response},
};

use crate::{http::ApiTags, state::AppState};

pub struct CacheApi;

pub fn api() -> impl OpenApi {
    CacheApi
}

#[OpenApi]
impl CacheApi {
    /// Fetch and cache a remote image
    ///
    /// GET /api/cache/image?url=...
    #[oai(path = "/cache/image", method = "get", tag = "ApiTags::Cache")]
    async fn cached_image(
        &self,
        state: Data<&AppState>,
        url: Query<String>,
    ) -> Result<Response<Binary<Vec<u8>>>> {
        let image = state.images.get(&url).await?;

        Ok(Response::new(Binary(image.bytes.to_vec()))
            .header(header::CONTENT_TYPE, image.content_type)
            .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
            .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
            .header(
                header::CONTENT_SECURITY_POLICY,
                "default-src 'none'; img-src 'self' data:",
            ))
    }
}
