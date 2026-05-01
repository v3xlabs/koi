use crate::{
    error::KoiError,
    models::{
        account::Account,
        asset::{Asset, identity::AssetIdentity},
    },
    state::AppState,
};
use chrono::{DateTime, Utc};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Object, Clone)]
pub struct AccountBalances {
    pub balances: Vec<AccountBalance>,
}

#[derive(Serialize, Deserialize, Object, Clone)]
pub struct AccountBalance {
    pub asset_identity: AssetIdentity,
    pub balance: String,
    pub updated_at: DateTime<Utc>,
}

impl Account {
    pub async fn get_balances(&self, state: &AppState) -> Result<AccountBalances, KoiError> {
        let assets = Self::get_assets(state, self.account_identity.clone()).await?;

        let mut balances = Vec::new();
        for asset in assets {
            let balance = Asset::fetch_balance(state, &asset, self).await?;
            balances.push(AccountBalance {
                asset_identity: asset,
                balance: balance.to_string(),
                updated_at: Utc::now(),
            });
        }

        Ok(AccountBalances { balances })
    }
}
