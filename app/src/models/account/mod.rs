use poem_openapi::{Object, Union};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::network::identity::NetworkIdentity;

pub mod extra;

#[derive(Serialize, Deserialize, Object, Clone)]
pub struct Account {
    pub account_id: Uuid,
    pub name: String,
    pub networks: Vec<NetworkIdentity>,
    pub metadata: WalletType,
}

#[derive(Serialize, Deserialize, Union, Clone)]
#[oai(discriminator_name = "type")]
#[serde(tag = "type")]
pub enum WalletType {
    Safe(SafeWallet),
    EOA(EOAWallet),
    View(ViewWallet),
    Railgun(RailgunWallet),
}

#[derive(Serialize, Deserialize, Object, Clone)]
pub struct SafeWallet {
    pub evm_address: String,
}

#[derive(Serialize, Deserialize, Object, Clone)]
pub struct EOAWallet {
    pub evm_address: String,
}

#[derive(Serialize, Deserialize, Object, Clone)]
pub struct ViewWallet {
    pub evm_address: String,
}

#[derive(Serialize, Deserialize, Object, Clone)]
pub struct RailgunWallet {
    pub railgun_address: String,
}
