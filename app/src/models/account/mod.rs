use poem_openapi::{Object, Union};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod extra;

#[derive(Serialize, Deserialize, Object, Clone)]
pub struct Account {
    pub account_id: Uuid,
    pub name: String,
    pub chains: Vec<String>,
    #[serde(flatten)]
    #[oai(flatten)]
    pub wallet: WalletType,
}

#[derive(Serialize, Deserialize, Union, Clone)]
#[oai(discriminator_name = "type")]
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
