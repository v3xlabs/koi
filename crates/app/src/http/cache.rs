use poem::{
    Error, Result,
    http::{StatusCode, header},
    web::Data,
};
use poem_openapi::{
    ApiResponse, OpenApi,
    param::Query,
    payload::{Binary, Response},
};

use crate::{http::ApiTags, state::AppState};

pub struct CacheApi;

#[derive(ApiResponse)]
enum StoreImageResponse {
    #[oai(status = 204)]
    Stored(#[oai(header = "Location")] String),
}

pub fn api() -> impl OpenApi {
    CacheApi
}

#[OpenApi]
impl CacheApi {
    /// Get a cached image
    ///
    /// GET /api/cache/image?id=...
    #[oai(path = "/cache/image", method = "get", tag = "ApiTags::Cache")]
    async fn cached_image(
        &self,
        state: Data<&AppState>,
        id: Query<String>,
    ) -> Result<Response<Binary<Vec<u8>>>> {
        let image = state
            .images
            .get(&id)
            .await?
            .ok_or_else(|| Error::from_status(StatusCode::NOT_FOUND))?;

        Ok(image_response(image))
    }

    /// Fetch a remote image and store it in the cache
    ///
    /// POST /api/cache/image?url=...
    #[oai(path = "/cache/image", method = "post", tag = "ApiTags::Cache")]
    async fn fetch_image(
        &self,
        state: Data<&AppState>,
        url: Query<String>,
    ) -> Result<StoreImageResponse> {
        let id = state.images.store(&url).await?;

        Ok(StoreImageResponse::Stored(format!(
            "/api/cache/image?id={id}"
        )))
    }
}

fn image_response(image: crate::models::image_cache::CachedImage) -> Response<Binary<Vec<u8>>> {
    Response::new(Binary(image.bytes.to_vec()))
        .header(header::CONTENT_TYPE, image.content_type)
        .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
        .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
        .header(
            header::CONTENT_SECURITY_POLICY,
            "default-src 'none'; img-src 'self' data:",
        )
}
