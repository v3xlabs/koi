use std::collections::HashMap;

use ratatui::{
    Frame,
    layout::{Rect, Size},
    style::{Color, Style},
    text::Span,
};
use ratatui_image::{
    FilterType, Image, Resize,
    picker::{Picker, ProtocolType},
    protocol::Protocol,
};

use crate::blo::blo_dynamic_image;
use koi::models::asset::{AssetIconColor, AssetIconColors};

const BLO_COLS: u16 = 8;
const BLO_HALFBLOCK_ROWS: u16 = 4;
const ICON_SCALE: u16 = 1;
const GRAPHICS_ICON_SCALE: u16 = 1;
/// Each terminal cell covers this many blo pixel columns/rows in list icons.
const LIST_BLO_PIXELS_PER_CELL: u16 = 2;

pub const ICON_HEIGHT: u16 = BLO_HALFBLOCK_ROWS * ICON_SCALE;
pub const LIST_ICON_HEIGHT: u16 = BLO_HALFBLOCK_ROWS / LIST_BLO_PIXELS_PER_CELL;
pub const LIST_ICON_WIDTH: u16 = BLO_COLS / LIST_BLO_PIXELS_PER_CELL;

pub struct IconRenderer {
    picker: Picker,
    cache: HashMap<(String, u16), Protocol>,
}

impl IconRenderer {
    pub fn new() -> Self {
        let picker = Picker::from_query_stdio().unwrap_or_else(|_| Picker::halfblocks());
        Self {
            picker,
            cache: HashMap::new(),
        }
    }

    pub fn uses_graphics(&self) -> bool {
        self.picker.protocol_type() != ProtocolType::Halfblocks
    }

    // The initial stdio query measured the cell pixel size once; a terminal zoom
    // changes it, so cached protocols would render at the wrong pixel dimensions.
    // Re-querying stdio here would race the input thread, so derive the cell size
    // from the window ioctl instead and re-encode everything.
    pub fn handle_resize(&mut self) {
        if !self.uses_graphics() {
            return;
        }
        let Ok(window) = crossterm::terminal::window_size() else {
            return;
        };
        if window.columns == 0 || window.rows == 0 {
            return;
        }
        let font = ratatui_image::FontSize::new(
            window.width / window.columns,
            window.height / window.rows,
        );
        if font.width == 0 || font.height == 0 {
            return;
        }
        let current = self.picker.font_size();
        if font.width == current.width && font.height == current.height {
            return;
        }
        let protocol_type = self.picker.protocol_type();
        #[allow(deprecated)]
        let mut picker = Picker::from_fontsize(font);
        picker.set_protocol_type(protocol_type);
        self.picker = picker;
        self.cache.clear();
    }

    pub fn icon_height() -> u16 {
        ICON_HEIGHT
    }

    pub fn list_row_height() -> u16 {
        LIST_ICON_HEIGHT
    }

    pub fn list_column_width() -> u16 {
        LIST_ICON_WIDTH
    }

    fn scale(&self) -> u16 {
        if self.uses_graphics() {
            GRAPHICS_ICON_SCALE
        } else {
            ICON_SCALE
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, address: &str, scale: u16) {
        self.render_sized(frame, area, address, scale);
    }

    pub fn render_list_icon(&mut self, frame: &mut Frame, area: Rect, address: &str) {
        if !self.uses_graphics() {
            return;
        }
        self.render_sized(frame, area, address, LIST_BLO_PIXELS_PER_CELL);
    }

    pub fn render_large(&mut self, frame: &mut Frame, area: Rect, address: &str) {
        if !self.uses_graphics() {
            return;
        }
        self.render_sized(frame, area, address, ICON_SCALE);
    }

    pub fn render_asset_icon(&mut self, frame: &mut Frame, area: Rect, asset_id: &str, png: &[u8]) {
        if !self.uses_graphics() || area.width == 0 || area.height == 0 {
            return;
        }
        let key = format!("asset:{asset_id}");
        if !self.cache.contains_key(&(key.clone(), 0)) {
            let Ok(image) = image::load_from_memory(png) else {
                return;
            };
            let Ok(protocol) = self.picker.new_protocol(
                image,
                Size::new(area.width, area.height),
                Resize::Scale(Some(FilterType::Nearest)),
            ) else {
                return;
            };
            self.cache.insert((key.clone(), 0), protocol);
        }
        if let Some(protocol) = self.cache.get(&(key, 0)) {
            frame.render_widget(Image::new(protocol), area);
        }
    }

    pub fn asset_symbol_spans(colors: &AssetIconColors) -> Vec<Span<'static>> {
        vec![Span::styled(
            "■",
            Style::default().fg(terminal_color(colors.primary)),
        )]
    }

    pub fn color_symbol(symbol: &str, colors: &AssetIconColors) -> Vec<Span<'static>> {
        let characters: Vec<_> = symbol.chars().collect();
        let Some(secondary) = colors.secondary.filter(|_| characters.len() > 2) else {
            return vec![Span::styled(
                symbol.to_string(),
                Style::default().fg(terminal_color(colors.primary)),
            )];
        };

        characters
            .into_iter()
            .enumerate()
            .map(|(index, character)| {
                let progress = index as f32 / (symbol.chars().count() - 1) as f32;
                let blend = |start: u8, end: u8| {
                    (start as f32 + (end as f32 - start as f32) * progress).round() as u8
                };
                let color = AssetIconColor {
                    red: blend(colors.primary.red, secondary.red),
                    green: blend(colors.primary.green, secondary.green),
                    blue: blend(colors.primary.blue, secondary.blue),
                };
                Span::styled(
                    character.to_string(),
                    Style::default().fg(terminal_color(color)),
                )
            })
            .collect()
    }

    fn render_sized(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        address: &str,
        blo_pixels_per_cell: u16,
    ) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        if !self.uses_graphics() {
            return;
        }

        let cells = icon_cells(blo_pixels_per_cell);
        let render_area = Rect {
            x: area.x,
            y: area.y,
            width: cells.width.min(area.width),
            height: cells.height.min(area.height),
        };

        if let Some(protocol) = self.get_graphics(address, blo_pixels_per_cell) {
            frame.render_widget(Image::new(protocol), render_area);
        }
    }

    fn get_graphics(&mut self, address: &str, blo_pixels_per_cell: u16) -> Option<&Protocol> {
        let key = (address.to_lowercase(), blo_pixels_per_cell);
        if !self.cache.contains_key(&key) {
            let cells = icon_cells(blo_pixels_per_cell);
            let side = icon_pixel_side(&self.picker, blo_pixels_per_cell);
            let image = blo_dynamic_image(address).resize_exact(side, side, FilterType::Nearest);
            let protocol = self
                .picker
                .new_protocol(image, cells, Resize::Scale(Some(FilterType::Nearest)))
                .ok()?;
            self.cache.insert(key.clone(), protocol);
        }
        self.cache.get(&key)
    }
}

fn terminal_color(color: koi::models::asset::AssetIconColor) -> Color {
    let term = std::env::var("TERM").unwrap_or_default();
    let color_term = std::env::var("COLORTERM").unwrap_or_default();
    if std::env::var_os("NO_COLOR").is_some() || term == "dumb" {
        return Color::Reset;
    }
    let luminance =
        (u16::from(color.red) * 54 + u16::from(color.green) * 183 + u16::from(color.blue) * 19)
            / 256;
    let target = luminance.clamp(72, 208);
    let adjust = |channel: u8| {
        if luminance == 0 {
            target as u8
        } else {
            (u16::from(channel) * target / luminance).min(255) as u8
        }
    };
    let red = adjust(color.red);
    let green = adjust(color.green);
    let blue = adjust(color.blue);
    if color_term.contains("truecolor")
        || color_term.contains("24bit")
        || term.contains("truecolor")
    {
        return Color::Rgb(red, green, blue);
    }
    if term.contains("color") {
        return Color::Indexed(16 + 36 * (red / 51) + 6 * (green / 51) + blue / 51);
    }
    Color::Reset
}

fn icon_cells(blo_pixels_per_cell: u16) -> Size {
    Size::new(
        BLO_COLS / blo_pixels_per_cell,
        BLO_HALFBLOCK_ROWS / blo_pixels_per_cell,
    )
}

fn icon_pixel_side(picker: &Picker, blo_pixels_per_cell: u16) -> u32 {
    (8 / blo_pixels_per_cell as u32) * picker.font_size().width as u32
}
