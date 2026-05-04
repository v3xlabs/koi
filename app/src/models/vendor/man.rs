use std::{collections::HashMap, sync::Mutex};

use sqlx::{SqlitePool, query, query_as};
use tracing::info;

use super::Vendor;
use super::flags::VendorFlag;
use crate::error::KoiError;

pub struct VendorManager {
    pub vendors: Mutex<HashMap<VendorFlag, bool>>,
}

impl VendorManager {
    pub async fn init(database: &SqlitePool) -> Result<Self, KoiError> {
        let vendors = query_as::<_, Vendor>("SELECT * FROM vendors")
            .fetch_all(database)
            .await
            .map_err(KoiError::from)?;

        let mut map = HashMap::new();
        for vendor in vendors {
            map.insert(vendor.vendor_flag, vendor.vendor_status);
        }

        info!("Initialized with vendors: {:?}", map);

        Ok(Self {
            vendors: Mutex::new(map),
        })
    }

    pub fn has_flag(&self, flag: VendorFlag) -> bool {
        self.vendors
            .lock()
            .expect("vendor mutex poisoned")
            .get(&flag)
            .copied()
            .unwrap_or(false)
    }

    // only enabled flags
    pub fn all(&self) -> Vec<VendorFlag> {
        self.vendors
            .lock()
            .expect("vendor mutex poisoned")
            .iter()
            .filter_map(|(key, enabled)| -> Option<VendorFlag> {
                match enabled {
                    true => Some(key.clone()),
                    false => None,
                }
            })
            .collect::<Vec<VendorFlag>>()
    }

    pub async fn set_flag(
        &self,
        flag: &VendorFlag,
        enabled: bool,
        database: &SqlitePool,
    ) -> Result<(), KoiError> {
        {
            let mut vendors = self.vendors.lock().expect("vendor mutex poisoned");
            vendors.insert(flag.clone(), enabled);
        }

        query("INSERT INTO vendors (vendor_flag, vendor_status) VALUES (?, ?) ON CONFLICT (vendor_flag) DO UPDATE SET vendor_status = ?")
            .bind(flag.to_string())
            .bind(enabled)
            .bind(enabled)
            .execute(database)
            .await
            .map_err(KoiError::from)?;

        Ok(())
    }
}
