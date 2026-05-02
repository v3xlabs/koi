use std::collections::HashMap;

use anyhow::{Context, Result};
use koi::models::{
    account::{balances::AccountBalances, Account},
    asset::Asset,
    network::{endpoint::NetworkEndpoint, pool::RpcPoolStats, Network},
    quoter::Quoter,
    tx::Tx,
    vendor::flags::{VendorFlag, VendorFlagInfo},
};
use reqwest::Client;
use serde::Deserialize;

const AUTH_TOKEN: &str = "hello";
const DISPLAY_CURRENCY: &str = "fiat:usd";

#[derive(Deserialize)]
struct AccountsResponse {
    accounts: Vec<Account>,
}

#[derive(Deserialize)]
struct NetworksResponse {
    networks: Vec<Network>,
}

#[derive(Deserialize)]
struct AssetsResponse {
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct QuotersResponse {
    quoters: Vec<Quoter>,
}

#[derive(Deserialize)]
struct VendorsResponse {
    vendors: Vec<VendorFlag>,
}

#[derive(Deserialize)]
struct VendorFlagInfoResponse {
    vendors: Vec<VendorFlagInfo>,
}

#[derive(Deserialize)]
struct AccountTxResponse {
    transactions: Vec<Tx>,
}

#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    base: String,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        let base = format!("{}/api", base_url.trim_end_matches('/'));
        Self {
            client: Client::new(),
            base,
        }
    }

    pub async fn health(&self) -> Result<()> {
        self.get::<String>("/health").await.map(|_| ())
    }

    pub async fn accounts(&self) -> Result<Vec<Account>> {
        let response: AccountsResponse = self.get("/acc").await?;
        Ok(response.accounts)
    }

    pub async fn account_balances(&self, account_identity: u64) -> Result<AccountBalances> {
        self.get(&format!(
            "/acc/{account_identity}/balances?display_currency={DISPLAY_CURRENCY}"
        ))
        .await
    }

    pub async fn account_transactions(&self, account_identity: u64) -> Result<Vec<Tx>> {
        let response: AccountTxResponse = self.get(&format!("/acc/{account_identity}/tx")).await?;
        Ok(response.transactions)
    }

    pub async fn networks(&self) -> Result<Vec<Network>> {
        let response: NetworksResponse = self.get("/net").await?;
        Ok(response.networks)
    }

    pub async fn network_rpc_stats(&self, network_identity: u64) -> Result<RpcPoolStats> {
        self.get(&format!("/net/{network_identity}/rpc")).await
    }

    pub async fn network_endpoints(
        &self,
        network_identity: u64,
    ) -> Result<Vec<NetworkEndpoint>> {
        self.get(&format!("/net/{network_identity}/endpoints")).await
    }

    pub async fn delete_network_endpoint(
        &self,
        network_identity: u64,
        endpoint_identity: i32,
    ) -> Result<()> {
        self.delete(&format!(
            "/net/{network_identity}/endpoints/{endpoint_identity}"
        ))
        .await
    }

    pub async fn assets(&self) -> Result<HashMap<String, Asset>> {
        let response: AssetsResponse = self.get("/asset").await?;
        Ok(response
            .assets
            .into_iter()
            .map(|asset| (asset.asset_identity.to_string(), asset))
            .collect())
    }

    pub async fn delete_asset(&self, asset_identity: &str) -> Result<()> {
        self.delete(&format!("/asset/{asset_identity}")).await
    }

    pub async fn quoters(&self) -> Result<Vec<Quoter>> {
        let response: QuotersResponse = self.get("/quoter").await?;
        Ok(response.quoters)
    }

    pub async fn vendors(&self) -> Result<Vec<VendorFlag>> {
        let response: VendorsResponse = self.get("/vendor").await?;
        Ok(response.vendors)
    }

    pub async fn all_vendors(&self) -> Result<Vec<VendorFlagInfo>> {
        let response: VendorFlagInfoResponse = self.get("/vendor/all").await?;
        Ok(response.vendors)
    }

    pub async fn set_vendor(&self, flag: &str, enabled: bool) -> Result<()> {
        if enabled {
            self.post_empty(&format!("/vendor/{flag}")).await
        } else {
            self.delete(&format!("/vendor/{flag}")).await
        }
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let url = format!("{}{path}", self.base);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {AUTH_TOKEN}"))
            .send()
            .await
            .with_context(|| format!("request failed: GET {url}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("GET {url} returned {status}: {body}");
        }

        response
            .json()
            .await
            .with_context(|| format!("failed to decode response from GET {url}"))
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let url = format!("{}{path}", self.base);
        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {AUTH_TOKEN}"))
            .send()
            .await
            .with_context(|| format!("request failed: DELETE {url}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("DELETE {url} returned {status}: {body}");
        }

        Ok(())
    }

    async fn post_empty(&self, path: &str) -> Result<()> {
        let url = format!("{}{path}", self.base);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {AUTH_TOKEN}"))
            .send()
            .await
            .with_context(|| format!("request failed: POST {url}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("POST {url} returned {status}: {body}");
        }

        Ok(())
    }
}
