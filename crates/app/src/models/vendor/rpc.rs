use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::flags::{VendorFlag, VendorFlagInfo};
use crate::{
    error::KoiError,
    rpc::{EmptyParams, RpcHandler},
    rpc_method,
    state::AppState,
};

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct VendorParams {
    pub flag: VendorFlag,
}

rpc_method!(VendorListEnabled, "vendor.listEnabled", EmptyParams => Vec<VendorFlag>);
rpc_method!(VendorListAll, "vendor.listAll", EmptyParams => Vec<VendorFlagInfo>);
rpc_method!(VendorEnable, "vendor.enable", VendorParams => ());
rpc_method!(VendorDisable, "vendor.disable", VendorParams => ());

impl RpcHandler for VendorListEnabled {
    async fn handle(state: &AppState, _params: EmptyParams) -> Result<Vec<VendorFlag>, KoiError> {
        Ok(state.vendors.all())
    }
}

impl RpcHandler for VendorListAll {
    async fn handle(
        _state: &AppState,
        _params: EmptyParams,
    ) -> Result<Vec<VendorFlagInfo>, KoiError> {
        Ok(VendorFlag::all())
    }
}

impl RpcHandler for VendorEnable {
    async fn handle(state: &AppState, params: VendorParams) -> Result<(), KoiError> {
        state
            .vendors
            .set_flag(&params.flag, true, &state.database)
            .await
    }
}

impl RpcHandler for VendorDisable {
    async fn handle(state: &AppState, params: VendorParams) -> Result<(), KoiError> {
        state
            .vendors
            .set_flag(&params.flag, false, &state.database)
            .await
    }
}
