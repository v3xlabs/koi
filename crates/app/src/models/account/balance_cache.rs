use std::sync::Arc;

use moka::future::Cache;

use super::{Account, balances::AccountBalances};
use crate::{
    error::KoiError,
    models::{
        account::identity::AccountIdentity,
        asset::identity::AssetIdentity,
    },
    state::AppState,
};

#[derive(Hash, Eq, PartialEq, Clone)]
struct BalanceCacheKey {
    account_identity: AccountIdentity,
    display_currency: AssetIdentity,
}

pub struct BalanceCacheManager {
    cache: Cache<BalanceCacheKey, AccountBalances>,
}

impl BalanceCacheManager {
    pub fn new() -> Self {
        Self {
            cache: Cache::builder().max_capacity(256).build(),
        }
    }

    pub async fn get_balances(
        &self,
        state: &AppState,
        account: &Account,
        display_currency: &AssetIdentity,
        fresh: bool,
    ) -> Result<AccountBalances, KoiError> {
        let key = BalanceCacheKey {
            account_identity: account.account_identity.clone(),
            display_currency: display_currency.clone(),
        };

        if !fresh {
            if let Some(cached) = self.cache.get(&key).await {
                return Ok(cached);
            }
        } else {
            self.cache.invalidate(&key).await;
        }

        let state = state.clone();
        let account = account.clone();
        let display_currency = display_currency.clone();

        self.cache
            .try_get_with(key, async move {
                account
                    .fetch_balances(&state, &display_currency)
                    .await
            })
            .await
            .map_err(|error| match Arc::try_unwrap(error) {
                Ok(error) => error,
                Err(error) => KoiError::Internal(error.to_string()),
            })
    }
}
