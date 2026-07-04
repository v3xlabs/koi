use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use bytes::Bytes;
use reqwest::{Client, Url, redirect::Policy};
use sha2::{Digest, Sha256};
use tokio::fs;

use crate::error::KoiError;

const MAX_IMAGE_BYTES: u64 = 2 * 1024 * 1024;
const USER_AGENT: &str = concat!("koi/", env!("CARGO_PKG_VERSION"), " image-cache");

const IMAGE_TYPES: &[ImageType] = &[
    ImageType {
        content_type: "image/avif",
        extension: "avif",
    },
    ImageType {
        content_type: "image/gif",
        extension: "gif",
    },
    ImageType {
        content_type: "image/jpeg",
        extension: "jpg",
    },
    ImageType {
        content_type: "image/png",
        extension: "png",
    },
    ImageType {
        content_type: "image/svg+xml",
        extension: "svg",
    },
    ImageType {
        content_type: "image/webp",
        extension: "webp",
    },
];

#[derive(Clone)]
pub struct CachedImage {
    pub bytes: Bytes,
    pub content_type: &'static str,
}

#[derive(Clone, Copy)]
struct ImageType {
    content_type: &'static str,
    extension: &'static str,
}

pub struct ImageCache {
    cache_dir: PathBuf,
}

impl ImageCache {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    pub async fn get(&self, raw_url: &str) -> Result<CachedImage, KoiError> {
        let url = validate_url(raw_url)?;
        let key = cache_key(url.as_str());

        if let Some(image) = self.read(&key).await? {
            return Ok(image);
        }

        self.fetch(&url, &key).await
    }

    async fn read(&self, key: &str) -> Result<Option<CachedImage>, KoiError> {
        for image_type in IMAGE_TYPES {
            let path = self.path(key, *image_type);
            let bytes = match fs::read(&path).await {
                Ok(bytes) => bytes,
                Err(err) if err.kind() == ErrorKind::NotFound => continue,
                Err(err) => {
                    return Err(KoiError::Internal(format!(
                        "failed to read image cache file {}: {err}",
                        path.display()
                    )));
                }
            };

            if bytes.len() as u64 <= MAX_IMAGE_BYTES {
                return Ok(Some(CachedImage {
                    bytes: Bytes::from(bytes),
                    content_type: image_type.content_type,
                }));
            }
        }

        Ok(None)
    }

    async fn fetch(&self, url: &Url, key: &str) -> Result<CachedImage, KoiError> {
        let client = Client::builder()
            .redirect(Policy::none())
            .timeout(Duration::from_secs(10))
            .user_agent(USER_AGENT)
            .build()
            .map_err(|err| KoiError::Internal(format!("failed to create image client: {err}")))?;

        let response = client
            .get(url.clone())
            .send()
            .await
            .map_err(|err| KoiError::Internal(format!("failed to fetch image: {err}")))?;

        if !response.status().is_success() {
            return Err(KoiError::Internal(format!(
                "image fetch failed with status {}",
                response.status()
            )));
        }

        if response.content_length().unwrap_or(0) > MAX_IMAGE_BYTES {
            return Err(KoiError::Internal("image is too large".to_string()));
        }

        let image_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .and_then(image_type_from_content_type)
            .ok_or_else(|| KoiError::Internal("response is not a supported image".to_string()))?;

        let bytes = response
            .bytes()
            .await
            .map_err(|err| KoiError::Internal(format!("failed to read image body: {err}")))?;

        if bytes.len() as u64 > MAX_IMAGE_BYTES {
            return Err(KoiError::Internal("image is too large".to_string()));
        }

        write_atomic(&self.path(key, image_type), &bytes).await?;

        Ok(CachedImage {
            bytes,
            content_type: image_type.content_type,
        })
    }

    fn path(&self, key: &str, image_type: ImageType) -> PathBuf {
        self.cache_dir
            .join(format!("{key}.{}", image_type.extension))
    }
}

fn validate_url(raw_url: &str) -> Result<Url, KoiError> {
    let url = Url::parse(raw_url)
        .map_err(|err| KoiError::Internal(format!("invalid image URL: {err}")))?;

    match url.scheme() {
        "http" | "https" => {}
        _ => {
            return Err(KoiError::Internal(
                "image URL must use http or https".to_string(),
            ));
        }
    }

    url.host_str()
        .ok_or_else(|| KoiError::Internal("image URL has no host".to_string()))?;

    Ok(url)
}

fn image_type_from_content_type(value: &str) -> Option<ImageType> {
    let value = value
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    IMAGE_TYPES
        .iter()
        .copied()
        .find(|image_type| image_type.content_type == value)
}

fn cache_key(url: &str) -> String {
    hex::encode(Sha256::digest(url.as_bytes()))
}

async fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), KoiError> {
    let parent = path.parent().ok_or_else(|| {
        KoiError::Internal(format!(
            "image cache path has no parent: {}",
            path.display()
        ))
    })?;
    fs::create_dir_all(parent).await.map_err(|err| {
        KoiError::Internal(format!(
            "failed to create image cache directory {}: {err}",
            parent.display()
        ))
    })?;

    let temp_path = temp_path(parent, path)?;
    fs::write(&temp_path, bytes).await.map_err(|err| {
        KoiError::Internal(format!(
            "failed to write image cache file {}: {err}",
            temp_path.display()
        ))
    })?;
    fs::rename(&temp_path, path).await.map_err(|err| {
        KoiError::Internal(format!(
            "failed to move image cache file {} to {}: {err}",
            temp_path.display(),
            path.display()
        ))
    })?;

    Ok(())
}

fn temp_path(parent: &Path, path: &Path) -> Result<PathBuf, KoiError> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            KoiError::Internal(format!(
                "image cache path has no file name: {}",
                path.display()
            ))
        })?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    Ok(parent.join(format!(".{file_name}.{}.{}.tmp", std::process::id(), nonce)))
}

#[cfg(test)]
mod tests {
    use super::{ImageCache, cache_key, image_type_from_content_type};

    #[test]
    fn cache_key_is_sha256_hex() {
        assert_eq!(
            cache_key("https://example.com/icon.png"),
            "4d2b6c4e8c53f5b51640c4e08a08b625b7437e866ee270599e3dcb61eedc028e"
        );
    }

    #[test]
    fn cache_path_uses_image_extension() {
        let cache = ImageCache::new("/tmp/koi-image-cache-test".into());
        let image_type = image_type_from_content_type("image/png").unwrap();
        let path = cache.path("abc123", image_type);
        assert_eq!(
            path.file_name().and_then(|name| name.to_str()),
            Some("abc123.png")
        );
    }

    #[test]
    fn content_type_matching_is_strict() {
        assert_eq!(
            image_type_from_content_type("image/png; charset=utf-8").map(|t| t.extension),
            Some("png")
        );
        assert!(image_type_from_content_type("text/html").is_none());
    }
}
