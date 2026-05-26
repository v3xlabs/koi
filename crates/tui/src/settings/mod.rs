use std::collections::{HashMap, HashSet};

use koi::models::{
    network::endpoint::NetworkEndpoint,
    quoter::Quoter,
    vendor::flags::VendorFlagInfo,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsSection {
    General,
    Networks,
    Assets,
    PriceFeeds,
    Vendors,
}

impl SettingsSection {
    pub const ALL: [SettingsSection; 2] = [SettingsSection::General, SettingsSection::Vendors];

    pub fn title(self) -> &'static str {
        match self {
            SettingsSection::General => "General",
            SettingsSection::Networks => "Networks",
            SettingsSection::Assets => "Assets",
            SettingsSection::PriceFeeds => "Prices",
            SettingsSection::Vendors => "Vendors",
        }
    }

}

pub struct SettingsState {
    pub section_index: usize,
    pub row_index: usize,
    pub row_scroll: usize,
    pub nested_network: Option<u64>,
    pub notice: Option<String>,
}

impl SettingsState {
    pub fn new() -> Self {
        Self {
            section_index: 0,
            row_index: 0,
            row_scroll: 0,
            nested_network: None,
            notice: None,
        }
    }

    pub fn section(&self) -> SettingsSection {
        SettingsSection::ALL[self.section_index.min(SettingsSection::ALL.len() - 1)]
    }

    pub fn move_section(&mut self, delta: i32) {
        let next = self.section_index as i32 + delta;
        self.section_index = next.clamp(0, SettingsSection::ALL.len() as i32 - 1) as usize;
        self.row_index = 0;
        self.row_scroll = 0;
        self.nested_network = None;
    }

    pub fn move_row(&mut self, delta: i32, len: usize) {
        if len == 0 {
            self.row_index = 0;
            return;
        }

        let next = self.row_index as i32 + delta;
        self.row_index = next.clamp(0, len as i32 - 1) as usize;
    }
}

pub struct SettingsSnapshot {
    pub endpoints: HashMap<u64, Vec<NetworkEndpoint>>,
    pub quoters: Vec<Quoter>,
    pub all_vendors: Vec<VendorFlagInfo>,
    pub enabled_vendors: HashSet<String>,
}

impl SettingsSnapshot {
    pub fn enabled_vendor_count(&self) -> usize {
        self.enabled_vendors.len()
    }
}
