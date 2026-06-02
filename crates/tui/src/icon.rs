use std::collections::HashMap;

use ratatui::{
    layout::{Rect, Size},
    Frame,
};
use ratatui_image::{
    FilterType, Image, Resize,
    picker::{Picker, ProtocolType},
    protocol::Protocol,
};

use crate::blo::blo_dynamic_image;

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

fn icon_cells(blo_pixels_per_cell: u16) -> Size {
    Size::new(
        BLO_COLS / blo_pixels_per_cell,
        BLO_HALFBLOCK_ROWS / blo_pixels_per_cell,
    )
}

fn icon_pixel_side(picker: &Picker, blo_pixels_per_cell: u16) -> u32 {
    (8 / blo_pixels_per_cell as u32) * picker.font_size().width as u32
}
