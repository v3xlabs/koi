use identity::AssetIdentity;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

use image::{ImageFormat, ImageReader};
use resvg::{tiny_skia, usvg};
use sqlx::{prelude::FromRow, query, query_as};
use ts_rs::TS;

use crate::{error::KoiError, models::network::identity::NetworkIdentity, state::DB};

pub mod balances;
pub mod erc20;
pub mod identity;
pub mod metadata;
pub mod rpc;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, TS)]
#[ts(optional_fields)]
pub struct Asset {
    pub asset_identity: AssetIdentity,
    pub asset_name: String,
    pub asset_symbol: String,
    pub asset_decimals: u8,
    pub asset_icon_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(optional_fields)]
pub struct AssetUpdate {
    pub asset_name: Option<String>,
    pub asset_symbol: Option<String>,
    pub asset_decimals: Option<u8>,
    pub asset_icon_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, TS)]
pub struct AssetIconColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(optional_fields)]
pub struct AssetIconColors {
    pub primary: AssetIconColor,
    pub secondary: Option<AssetIconColor>,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
pub struct AssetIconData {
    pub png_data: Vec<u8>,
    pub colors: AssetIconColors,
}

#[derive(FromRow)]
struct CachedAssetIcon {
    png_data: Vec<u8>,
    primary_red: Option<i64>,
    primary_green: Option<i64>,
    primary_blue: Option<i64>,
    secondary_red: Option<i64>,
    secondary_green: Option<i64>,
    secondary_blue: Option<i64>,
}

impl Asset {
    pub async fn icon(
        database: &DB,
        asset_identity: &AssetIdentity,
    ) -> Result<Option<AssetIconData>, KoiError> {
        if let Some(icon) = query_as::<_, CachedAssetIcon>("SELECT png_data, primary_red, primary_green, primary_blue, secondary_red, secondary_green, secondary_blue FROM asset_icons WHERE asset_identity = ?")
            .bind(asset_identity)
            .fetch_optional(database)
            .await?
        {
            let colors = cached_icon_colors(&icon).unwrap_or_else(|| {
                let colors = image::load_from_memory(&icon.png_data)
                    .map(|image| analyze_icon_colors(&image))
                    .unwrap_or_default();
                (colors, true)
            });
            if colors.1 {
                cache_icon_colors(database, asset_identity, &colors.0).await?;
            }
            return Ok(Some(AssetIconData {
                png_data: icon.png_data,
                colors: colors.0,
            }));
        }

        let asset = Self::get_by_id(database, asset_identity).await?;
        let Some(url) = asset.asset_icon_url else {
            return Ok(None);
        };
        let parsed = url::Url::parse(&url)
            .map_err(|_| KoiError::InvalidInput("asset icon URL is invalid".to_string()))?;
        if parsed.scheme() != "https" {
            return Err(KoiError::InvalidInput(
                "asset icon URL must use HTTPS".to_string(),
            ));
        }

        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|error| KoiError::Internal(format!("asset icon client: {error}")))?;
        let mut response = client.get(parsed).send().await.map_err(|error| {
            KoiError::Unavailable(format!("asset icon request failed: {error}"))
        })?;
        if !response.status().is_success() {
            return Err(KoiError::Unavailable(format!(
                "asset icon request failed with {}",
                response.status()
            )));
        }
        if response
            .content_length()
            .is_some_and(|length| length > 1_000_000)
        {
            return Err(KoiError::InvalidInput(
                "asset icon is too large".to_string(),
            ));
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|error| {
            KoiError::Unavailable(format!("asset icon download failed: {error}"))
        })? {
            if bytes.len().saturating_add(chunk.len()) > 1_000_000 {
                return Err(KoiError::InvalidInput(
                    "asset icon is too large".to_string(),
                ));
            }
            bytes.extend_from_slice(&chunk);
        }
        let (png_data, colors) = normalize_icon(&bytes)?;
        query("INSERT INTO asset_icons (asset_identity, png_data, primary_red, primary_green, primary_blue, secondary_red, secondary_green, secondary_blue) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(asset_identity)
            .bind(&png_data)
            .bind(colors.primary.red)
            .bind(colors.primary.green)
            .bind(colors.primary.blue)
            .bind(colors.secondary.map(|color| color.red))
            .bind(colors.secondary.map(|color| color.green))
            .bind(colors.secondary.map(|color| color.blue))
            .execute(database)
            .await?;
        Ok(Some(AssetIconData { png_data, colors }))
    }

    pub async fn all(database: &DB) -> Result<Vec<Asset>, KoiError> {
        query_as::<_, Asset>("SELECT * FROM assets")
            .fetch_all(database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn get_by_id(
        database: &DB,
        asset_identity: &AssetIdentity,
    ) -> Result<Asset, KoiError> {
        query_as::<_, Asset>("SELECT * FROM assets WHERE asset_identity = ?")
            .bind(asset_identity)
            .fetch_one(database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn get_by_network_id(
        database: &DB,
        network_identity: &NetworkIdentity,
    ) -> Result<Vec<Asset>, KoiError> {
        let pattern = format!("erc20:{}:%", network_identity);
        let pattern2 = format!("native:{}", network_identity);

        query_as::<_, Asset>(
            "SELECT * FROM assets WHERE asset_identity LIKE ? OR asset_identity = ?",
        )
        .bind(&pattern)
        .bind(&pattern2)
        .fetch_all(database)
        .await
        .map_err(KoiError::from)
    }

    pub async fn create(database: &DB, asset: Asset) -> Result<Asset, KoiError> {
        query_as::<_, Asset>("INSERT INTO assets (asset_identity, asset_name, asset_symbol, asset_decimals, asset_icon_url) VALUES (?, ?, ?, ?, ?) RETURNING *")
            .bind(asset.asset_identity)
            .bind(asset.asset_name)
            .bind(asset.asset_symbol)
            .bind(asset.asset_decimals)
            .bind(asset.asset_icon_url)
            .fetch_one(database)
            .await
            .map_err(KoiError::from)
    }

    pub async fn update(
        database: &DB,
        asset_identity: &AssetIdentity,
        asset: AssetUpdate,
    ) -> Result<Asset, KoiError> {
        let updated = query_as::<_, Asset>("UPDATE assets SET asset_name = ?, asset_symbol = ?, asset_decimals = ?, asset_icon_url = ? WHERE asset_identity = ? RETURNING *")
            .bind(asset.asset_name)
            .bind(asset.asset_symbol)
            .bind(asset.asset_decimals)
            .bind(asset.asset_icon_url)
            .bind(asset_identity)
            .fetch_one(database)
            .await
            .map_err(KoiError::from)?;
        query("DELETE FROM asset_icons WHERE asset_identity = ?")
            .bind(asset_identity)
            .execute(database)
            .await?;
        Ok(updated)
    }

    pub async fn delete(database: &DB, asset_identity: &AssetIdentity) -> Result<(), KoiError> {
        query("DELETE FROM assets WHERE asset_identity = ?")
            .bind(asset_identity)
            .execute(database)
            .await
            .map_err(KoiError::from)
            .map(|_| ())
    }
}

fn cached_icon_colors(icon: &CachedAssetIcon) -> Option<(AssetIconColors, bool)> {
    let primary = AssetIconColor {
        red: u8::try_from(icon.primary_red?).ok()?,
        green: u8::try_from(icon.primary_green?).ok()?,
        blue: u8::try_from(icon.primary_blue?).ok()?,
    };
    let secondary = match (
        icon.secondary_red,
        icon.secondary_green,
        icon.secondary_blue,
    ) {
        (None, None, None) => None,
        (Some(red), Some(green), Some(blue)) => Some(AssetIconColor {
            red: u8::try_from(red).ok()?,
            green: u8::try_from(green).ok()?,
            blue: u8::try_from(blue).ok()?,
        }),
        _ => return None,
    };
    Some((AssetIconColors { primary, secondary }, false))
}

async fn cache_icon_colors(
    database: &DB,
    asset_identity: &AssetIdentity,
    colors: &AssetIconColors,
) -> Result<(), KoiError> {
    query("UPDATE asset_icons SET primary_red = ?, primary_green = ?, primary_blue = ?, secondary_red = ?, secondary_green = ?, secondary_blue = ? WHERE asset_identity = ?")
        .bind(colors.primary.red)
        .bind(colors.primary.green)
        .bind(colors.primary.blue)
        .bind(colors.secondary.map(|color| color.red))
        .bind(colors.secondary.map(|color| color.green))
        .bind(colors.secondary.map(|color| color.blue))
        .bind(asset_identity)
        .execute(database)
        .await?;
    Ok(())
}

fn normalize_icon(bytes: &[u8]) -> Result<(Vec<u8>, AssetIconColors), KoiError> {
    let image = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|error| KoiError::InvalidInput(format!("asset icon format: {error}")))
        .and_then(|reader| {
            reader
                .decode()
                .map_err(|error| KoiError::InvalidInput(format!("asset icon: {error}")))
        })
        .or_else(|_| rasterize_svg(bytes))?
        .thumbnail(64, 64);
    let colors = analyze_icon_colors(&image);
    let mut output = Cursor::new(Vec::new());
    image
        .write_to(&mut output, ImageFormat::Png)
        .map_err(|error| KoiError::Internal(format!("asset icon encoding: {error}")))?;
    Ok((output.into_inner(), colors))
}

fn analyze_icon_colors(image: &image::DynamicImage) -> AssetIconColors {
    let pixels: Vec<_> = image
        .to_rgba8()
        .pixels()
        .filter_map(|pixel| {
            let [red, green, blue, alpha] = pixel.0;
            (alpha >= 16).then_some((AssetIconColor { red, green, blue }, u32::from(alpha)))
        })
        .collect();
    let Some((mut primary, _)) = weighted_mean(&pixels) else {
        return AssetIconColors::default();
    };
    let secondary_seed = pixels
        .iter()
        .max_by_key(|(color, _)| color_distance_squared(*color, primary))
        .map(|(color, _)| *color)
        .unwrap_or(primary);
    let mut secondary = secondary_seed;
    let mut primary_weight = 0;
    let mut secondary_weight = 0;

    for _ in 0..6 {
        let mut primary_pixels = Vec::new();
        let mut secondary_pixels = Vec::new();
        for &(color, weight) in &pixels {
            if color_distance_squared(color, primary) <= color_distance_squared(color, secondary) {
                primary_pixels.push((color, weight));
            } else {
                secondary_pixels.push((color, weight));
            }
        }
        primary_weight = primary_pixels.iter().map(|(_, weight)| *weight).sum();
        secondary_weight = secondary_pixels.iter().map(|(_, weight)| *weight).sum();
        if let Some((color, _)) = weighted_mean(&primary_pixels) {
            primary = color;
        }
        if let Some((color, _)) = weighted_mean(&secondary_pixels) {
            secondary = color;
        }
    }

    if secondary_weight > primary_weight {
        std::mem::swap(&mut primary, &mut secondary);
        std::mem::swap(&mut primary_weight, &mut secondary_weight);
    }
    let total_weight = primary_weight + secondary_weight;
    let varied = total_weight > 0
        && secondary_weight * 100 >= total_weight * 15
        && color_distance_squared(primary, secondary) >= 36 * 36
        && primary.chroma().max(secondary.chroma()) >= 24;
    AssetIconColors {
        primary,
        secondary: varied.then_some(secondary),
    }
}

fn weighted_mean(pixels: &[(AssetIconColor, u32)]) -> Option<(AssetIconColor, u32)> {
    let total: u32 = pixels.iter().map(|(_, weight)| *weight).sum();
    (total > 0).then(|| {
        let red = pixels
            .iter()
            .map(|(color, weight)| u32::from(color.red) * weight)
            .sum::<u32>()
            / total;
        let green = pixels
            .iter()
            .map(|(color, weight)| u32::from(color.green) * weight)
            .sum::<u32>()
            / total;
        let blue = pixels
            .iter()
            .map(|(color, weight)| u32::from(color.blue) * weight)
            .sum::<u32>()
            / total;
        (
            AssetIconColor {
                red: red as u8,
                green: green as u8,
                blue: blue as u8,
            },
            total,
        )
    })
}

fn color_distance_squared(left: AssetIconColor, right: AssetIconColor) -> u32 {
    let red = i32::from(left.red) - i32::from(right.red);
    let green = i32::from(left.green) - i32::from(right.green);
    let blue = i32::from(left.blue) - i32::from(right.blue);
    (red * red + green * green + blue * blue) as u32
}

impl AssetIconColor {
    fn chroma(self) -> u8 {
        self.red.max(self.green).max(self.blue) - self.red.min(self.green).min(self.blue)
    }
}

impl Default for AssetIconColors {
    fn default() -> Self {
        Self {
            primary: AssetIconColor {
                red: 128,
                green: 128,
                blue: 128,
            },
            secondary: None,
        }
    }
}

fn rasterize_svg(bytes: &[u8]) -> Result<image::DynamicImage, KoiError> {
    let tree = usvg::Tree::from_data(bytes, &usvg::Options::default())
        .map_err(|error| KoiError::InvalidInput(format!("asset SVG: {error}")))?;
    let mut pixmap = tiny_skia::Pixmap::new(64, 64)
        .ok_or_else(|| KoiError::Internal("could not allocate asset icon".to_string()))?;
    let size = tree.size();
    let scale = (64.0 / size.width()).min(64.0 / size.height());
    let transform = tiny_skia::Transform::from_scale(scale, scale).post_translate(
        (64.0 - size.width() * scale) / 2.0,
        (64.0 - size.height() * scale) / 2.0,
    );
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    image::RgbaImage::from_raw(64, 64, pixmap.take())
        .map(image::DynamicImage::ImageRgba8)
        .ok_or_else(|| KoiError::Internal("asset SVG rendering failed".to_string()))
}

#[cfg(test)]
mod tests {
    use super::{AssetIconColor, analyze_icon_colors, normalize_icon};

    #[test]
    fn normalizes_svg_to_bounded_png() {
        let (png, _) = normalize_icon(
            br##"<svg xmlns="http://www.w3.org/2000/svg" width="128" height="64"><rect width="128" height="64" fill="#0af"/></svg>"##,
        )
        .unwrap();
        let icon = image::load_from_memory(&png).unwrap();

        assert_eq!(icon.width(), 64);
        assert_eq!(icon.height(), 64);
    }

    #[test]
    fn extracts_two_colors_from_a_multicolor_icon() {
        let image = image::DynamicImage::ImageRgba8(image::RgbaImage::from_fn(64, 64, |x, _| {
            if x < 32 {
                image::Rgba([0, 170, 255, 255])
            } else {
                image::Rgba([255, 80, 80, 255])
            }
        }));

        let colors = analyze_icon_colors(&image);

        assert_eq!(
            colors.primary,
            AssetIconColor {
                red: 0,
                green: 170,
                blue: 255
            }
        );
        assert_eq!(
            colors.secondary,
            Some(AssetIconColor {
                red: 255,
                green: 80,
                blue: 80
            })
        );
    }

    #[test]
    fn suppresses_secondary_for_near_monochrome_icons() {
        let image = image::DynamicImage::ImageRgba8(image::RgbaImage::from_fn(64, 64, |x, _| {
            let value = if x < 32 { 80 } else { 160 };
            image::Rgba([value, value, value, 255])
        }));

        assert!(analyze_icon_colors(&image).secondary.is_none());
    }
}
