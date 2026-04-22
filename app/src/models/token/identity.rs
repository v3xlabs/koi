use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

use crate::models::network::identity::NetworkIdentity;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TokenIdentity {
    Native(NetworkIdentity),
    ERC20(NetworkIdentity, Address),
    // Fiat Currency
    // ISO 4217 Code
    Fiat(String),
}
