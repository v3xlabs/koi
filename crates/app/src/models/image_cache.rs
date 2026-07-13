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

    pub async fn get(&self, key: &str) -> Result<Option<CachedImage>, KoiError> {
        validate_cache_key(key)?;
        self.read(key).await
    }

    pub async fn store(&self, raw_url: &str) -> Result<String, KoiError> {
        let url = validate_url(raw_url)?;
        let key = cache_key(url.as_str());

        if self.read(&key).await?.is_none() {
            self.fetch_and_store(&url, &key).await?;
        }

        Ok(key)
    }

    pub async fn store_reference(&self, reference: &str) -> Result<String, KoiError> {
        if reference.starts_with("data:image/") {
            return Ok(reference.to_string());
        }

        if let Some(key) = reference.strip_prefix("/api/cache/image?id=") {
            validate_cache_key(key)?;
            return Ok(key.to_string());
        }

        if validate_cache_key(reference).is_ok() {
            return Ok(reference.to_string());
        }

        self.store(reference).await
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

    async fn fetch_and_store(&self, url: &Url, key: &str) -> Result<(), KoiError> {
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

        Ok(())
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

fn validate_cache_key(key: &str) -> Result<(), KoiError> {
    if key.len() == 64 && key.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Ok(());
    }

    Err(KoiError::Internal("invalid image cache id".to_string()))
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
    use super::{cache_key, validate_cache_key};

    #[test]
    fn cache_key_is_sha256_hex() {
        assert_eq!(
            cache_key("https://example.com/icon.png"),
            "4d2b6c4e8c53f5b51640c4e08a08b625b7437e866ee270599e3dcb61eedc028e"
        );
    }

    #[test]
    fn cache_key_validation_rejects_paths_and_urls() {
        assert!(validate_cache_key(&"a".repeat(64)).is_ok());
        assert!(validate_cache_key("../image").is_err());
        assert!(validate_cache_key("https://example.com/icon.png").is_err());
    }
}
