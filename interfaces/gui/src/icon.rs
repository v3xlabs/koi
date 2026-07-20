use anyhow::Result;
use std::io::Cursor;
use tao::window::Icon as WindowIcon;
use tray_icon::Icon as TrayIconImage;
#[cfg(not(target_os = "linux"))]
use wry::{PageLoadEvent, WebView, WebViewBuilder};

const KOI_ICON_PNG: &[u8] = include_bytes!("../../web/public/favicon_x64.png");

pub fn koi_window_icon() -> Result<WindowIcon> {
    let (rgba, width, height) = load_koi_icon_rgba()?;
    Ok(WindowIcon::from_rgba(rgba, width, height)?)
}

pub fn koi_tray_icon() -> Result<TrayIconImage> {
    let (rgba, width, height) = load_koi_icon_rgba()?;
    Ok(TrayIconImage::from_rgba(rgba, width, height)?)
}

pub fn load_koi_icon_rgba() -> Result<(Vec<u8>, u32, u32)> {
    let decoder = png::Decoder::new(Cursor::new(KOI_ICON_PNG));
    let mut reader = decoder.read_info()?;
    let output_buffer_size = reader
        .output_buffer_size()
        .ok_or_else(|| anyhow::anyhow!("invalid Koi icon dimensions"))?;
    let mut buffer = vec![0; output_buffer_size];
    let info = reader.next_frame(&mut buffer)?;
    let bytes = &buffer[..info.buffer_size()];

    let rgba = match info.color_type {
        png::ColorType::Rgba => bytes.to_vec(),
        png::ColorType::Rgb => bytes
            .chunks_exact(3)
            .flat_map(|pixel| [pixel[0], pixel[1], pixel[2], 0xff])
            .collect(),
        other => anyhow::bail!("unsupported Koi icon color type: {other:?}"),
    };

    Ok((rgba, info.width, info.height))
}
