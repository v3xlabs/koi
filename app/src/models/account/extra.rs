use uuid::Uuid;

use crate::{
    models::account::{EOAWallet, WalletType},
    error::KoiError,
    state::AppState,
};

use super::Account;

fn get_accounts() -> Vec<Account> {
    vec![Account {
        account_id: "4f8b9a49-5de4-4209-b1b9-6b2b5f085463".parse().unwrap(),
        name: "Wallet 1".to_string(),
        chains: vec!["Ethereum".to_string()],
        wallet: WalletType::EOA(EOAWallet {
            evm_address: "0x225f137127d9067788314bc7fcc1f36746a3c3B5".to_string(),
        }),
    }]
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
