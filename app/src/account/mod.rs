use poem_openapi::{Object, Union};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Object)]
pub struct Account {
    pub account_id: Uuid,
    pub name: String,
    pub chains: Vec<String>,
    #[oai(flatten)]
    #[serde(flatten)]
    pub wallet: WalletType,
}

#[derive(Serialize, Deserialize, Union)]
#[oai(discriminator_name = "type")]
pub enum WalletType {
    Safe(SafeWallet),
    EOA(EOAWallet),
    View(ViewWallet),
    Railgun(RailgunWallet),
}

#[derive(Serialize, Deserialize, Object)]
pub struct SafeWallet {
    pub evm_address: String,
}

#[derive(Serialize, Deserialize, Object)]
pub struct EOAWallet {
    pub evm_address: String,
}

#[derive(Serialize, Deserialize, Object)]
pub struct ViewWallet {
    pub evm_address: String,
}

#[derive(Serialize, Deserialize, Object)]
pub struct RailgunWallet {
    pub railgun_address: String,
}
