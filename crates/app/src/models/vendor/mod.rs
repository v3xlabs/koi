use flags::VendorFlag;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

pub mod flags;
pub mod man;
pub mod rpc;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Vendor {
    pub vendor_flag: VendorFlag,
    pub vendor_status: bool,
}
