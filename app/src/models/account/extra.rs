use uuid::Uuid;

use crate::{
    error::KoiError,
    models::{
        account::{EOAWallet, RailgunWallet, SafeWallet, ViewWallet, WalletType},
        network::identity::NetworkIdentity,
    },
    state::AppState,
};

use super::Account;

fn get_accounts() -> Vec<Account> {
    vec![
        Account {
            account_id: "4f8b9a49-5de4-4209-b1b9-6b2b5f085463".parse().unwrap(),
            name: "Wallet 1".to_string(),
            networks: vec![NetworkIdentity(1)],
            metadata: WalletType::EOA(EOAWallet {
                evm_address: "0x225f137127d9067788314bc7fcc1f36746a3c3B5".to_string(),
            }),
        },
        Account {
            account_id: "4f8b9a49-5de4-4209-b1b9-6b2b5f085464".parse().unwrap(),
            name: "Wallet 2".to_string(),
            networks: vec![NetworkIdentity(1)],
            metadata: WalletType::View(ViewWallet {
                evm_address: "0x8F8f07b6D61806Ec38febd15B07528dCF2903Ae7".to_string(),
            }),
        },
        Account {
            account_id: "4f8b9a49-5de4-4209-b1b9-6b2b5f085465".parse().unwrap(),
            name: "Wallet 3".to_string(),
            networks: vec![NetworkIdentity(1)],
            metadata: WalletType::Safe(SafeWallet {
                evm_address: "0xAC3EBDC2Dc0Cc20e937C970D46e2A232d3151aef".to_string(),
            }),
        },
        Account {
            account_id: "4f8b9a49-5de4-4209-b1b9-6b2b5f085466".parse().unwrap(),
            name: "Wallet 4".to_string(),
            networks: vec![NetworkIdentity(1)],
            metadata: WalletType::Railgun(RailgunWallet {
                railgun_address: "0zk225f137127d9067788314bc7fcc1f36746a3c3B5".to_string(),
            }),
        },
    ]
}

impl Account {
    pub async fn all(_state: &AppState) -> Result<Vec<Account>, KoiError> {
        Ok(get_accounts())
    }

    pub async fn get_by_id(_state: &AppState, account_id: Uuid) -> Result<Account, KoiError> {
        let accounts = get_accounts();
        let account = accounts
            .iter()
            .find(|account| account.account_id == account_id)
            .ok_or(KoiError::Internal(format!(
                "Account not found: {}",
                account_id
            )))?;

        Ok(account.to_owned())
    }
}
