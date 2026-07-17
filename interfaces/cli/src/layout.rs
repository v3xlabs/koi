use ratatui::layout::Rect;

use crate::app::{AccountPanel, Tab};

const TABLE_HEADER_ROWS: u16 = 2;

#[derive(Clone, Copy)]
pub struct ListTableLayout {
    pub area: Rect,
    pub scroll: usize,
    pub len: usize,
    pub row_height: u16,
}

impl Default for ListTableLayout {
    fn default() -> Self {
        Self {
            area: Rect::default(),
            scroll: 0,
            len: 0,
            row_height: 1,
        }
    }
}

#[derive(Clone, Copy)]
pub struct AccountSidebarLayout {
    pub area: Rect,
    pub panel_rows: [(Rect, AccountPanel); 4],
}

impl Default for AccountSidebarLayout {
    fn default() -> Self {
        Self {
            area: Rect::default(),
            panel_rows: [
                (Rect::default(), AccountPanel::Overview),
                (Rect::default(), AccountPanel::Assets),
                (Rect::default(), AccountPanel::Defi),
                (Rect::default(), AccountPanel::Transactions),
            ],
        }
    }
}

#[derive(Default, Clone)]
pub struct UiLayout {
    pub body: Rect,
    pub nav: Rect,
    pub list_table: Option<ListTableLayout>,
    pub account_sidebar: Option<AccountSidebarLayout>,
}

impl UiLayout {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn tab_at(&self, column: u16, row: u16) -> Option<Tab> {
        if !contains(self.nav, column, row) {
            return None;
        }

        let rel = column.saturating_sub(self.nav.x);
        let slot = u16::from(rel.saturating_mul(5) / self.nav.width.max(1)).min(4);
        Some(match slot {
            0 => Tab::Accounts,
            1 => Tab::Assets,
            2 => Tab::Prices,
            3 => Tab::Networks,
            _ => Tab::Settings,
        })
    }

    pub fn list_row_at(&self, column: u16, row: u16) -> Option<usize> {
        let table = self.list_table?;
        if !contains(table.area, column, row) {
            return None;
        }

        let data_top = table.area.y + TABLE_HEADER_ROWS;
        if row < data_top || row >= table.area.bottom().saturating_sub(1) {
            return None;
        }

        let row_height = table.row_height.max(1);
        let visible_row = usize::from(row.saturating_sub(data_top) / row_height);
        let index = table.scroll + visible_row;
        (index < table.len).then_some(index)
    }

    pub fn account_panel_at(&self, column: u16, row: u16) -> Option<AccountPanel> {
        let sidebar = self.account_sidebar?;
        if !contains(sidebar.area, column, row) {
            return None;
        }

        sidebar
            .panel_rows
            .iter()
            .find_map(|(rect, panel)| contains(*rect, column, row).then_some(*panel))
    }
}

pub fn contains(rect: Rect, column: u16, row: u16) -> bool {
    column >= rect.x && column < rect.right() && row >= rect.y && row < rect.bottom()
}

pub fn table_body_height(area: Rect) -> usize {
    area.height.saturating_sub(TABLE_HEADER_ROWS + 1) as usize
}

pub fn table_visible_rows(area: Rect, row_height: u16) -> usize {
    table_body_height(area) / row_height.max(1) as usize
}
