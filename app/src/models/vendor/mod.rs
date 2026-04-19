use std::collections::HashMap;

pub mod flags;

#[derive(Default)]
pub struct VendorManager {
    pub vendors: HashMap<String, bool>
}

impl VendorManager {
    pub fn has_flag(&self, flag: &str) -> bool {
        self.vendors.get(flag).copied().unwrap_or(false)
    }
}
