use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

const AAVE_ENDPOINT: &str = "https://api.v3.aave.com/graphql";
const MORPHO_ENDPOINT: &str = "https://blue-api.morpho.org/graphql";

const SUPPORTED_CHAINS: &[(u64, &str)] = &[
    (1, "Ethereum"),
    (42161, "Arbitrum"),
    (10, "Optimism"),
    (137, "Polygon"),
    (8453, "Base"),
];

#[derive(Clone, Debug)]
pub struct DefiPosition {
    pub protocol: String,
    pub chain_id: u64,
    pub chain_name: String,
    pub name: String,
    pub underlying_symbol: String,
    pub value: f64,
    pub value_usd: f64,
    pub tvl_usd: f64,
    pub apr: f64,
    pub earned_7d: f64,
    pub earned_7d_usd: f64,
}

#[derive(Clone, Debug)]
pub struct DefiResult {
    pub positions: Vec<DefiPosition>,
    pub errors: Vec<String>,
}

#[derive(Clone)]
pub struct DefiClient {
    client: Client,
}

impl DefiClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn positions(&self, holder: &str) -> DefiResult {
        let (aave, morpho) = tokio::join!(self.fetch_aave(holder), self.fetch_morpho(holder));

        let mut positions = Vec::new();
        let mut errors = Vec::new();

        match aave {
            Ok(mut result) => positions.append(&mut result),
            Err(error) => errors.push(format!("AAVE v3: {error:#}")),
        }

        match morpho {
            Ok(mut result) => positions.append(&mut result),
            Err(error) => errors.push(format!("Morpho: {error:#}")),
        }

        positions.sort_by(|left, right| {
            right
                .value_usd
                .partial_cmp(&left.value_usd)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        DefiResult { positions, errors }
    }

    async fn fetch_aave(&self, holder: &str) -> Result<Vec<DefiPosition>> {
        let markets = self.aave_markets().await?;
        if markets.is_empty() {
            return Ok(Vec::new());
        }

        let market_inputs = markets
            .iter()
            .map(|market| json!({ "chainId": market.chain_id, "address": market.address }))
            .collect::<Vec<_>>();

        let response: AaveSuppliesResponse = self
            .post_graphql(
                AAVE_ENDPOINT,
                AAVE_USER_SUPPLIES_QUERY,
                json!({
                    "markets": market_inputs,
                    "user": holder,
                }),
            )
            .await?;

        let mut positions = Vec::new();
        for supply in response.user_supplies {
            let value = parse_f64(&supply.balance.amount.value);
            if value == 0.0 {
                continue;
            }

            let value_usd = parse_f64(&supply.balance.usd);
            if value_usd > 0.0 && value_usd < 0.01 {
                continue;
            }

            let apr = parse_f64(&supply.apy.value);
            let earned_7d = value * apr * 7.0 / 365.0;
            let earned_7d_usd = if value > 0.0 {
                earned_7d * (value_usd / value)
            } else {
                0.0
            };

            positions.push(DefiPosition {
                protocol: "AAVE v3".to_string(),
                chain_id: supply.market.chain.chain_id,
                chain_name: supply.market.chain.name,
                name: format!("AAVE v3 {} supply", supply.currency.symbol),
                underlying_symbol: supply.currency.symbol,
                value,
                value_usd,
                tvl_usd: 0.0,
                apr,
                earned_7d,
                earned_7d_usd,
            });
        }

        Ok(positions)
    }

    async fn aave_markets(&self) -> Result<Vec<AaveMarket>> {
        let chain_ids = SUPPORTED_CHAINS
            .iter()
            .map(|(chain_id, _)| *chain_id)
            .collect::<Vec<_>>();
        let response: AaveMarketsResponse = self
            .post_graphql(
                AAVE_ENDPOINT,
                AAVE_MARKETS_QUERY,
                json!({ "chainIds": chain_ids }),
            )
            .await?;

        Ok(response
            .markets
            .into_iter()
            .map(|market| AaveMarket {
                address: market.address,
                chain_id: market.chain.chain_id,
            })
            .collect())
    }

    async fn fetch_morpho(&self, holder: &str) -> Result<Vec<DefiPosition>> {
        let mut tasks = Vec::new();
        for (chain_id, chain_name) in SUPPORTED_CHAINS {
            tasks.push(self.fetch_morpho_chain(holder, *chain_id, (*chain_name).to_string()));
        }

        let mut positions = Vec::new();
        let mut first_error = None;
        for task in tasks {
            match task.await {
                Ok(mut chain_positions) => positions.append(&mut chain_positions),
                Err(error) if first_error.is_none() => first_error = Some(error),
                Err(_) => {}
            }
        }

        if positions.is_empty() {
            if let Some(error) = first_error {
                return Err(error);
            }
        }

        Ok(positions)
    }

    async fn fetch_morpho_chain(
        &self,
        holder: &str,
        chain_id: u64,
        chain_name: String,
    ) -> Result<Vec<DefiPosition>> {
        let response: MorphoSnapshotResponse = self
            .post_graphql(
                MORPHO_ENDPOINT,
                MORPHO_POSITIONS_QUERY,
                json!({
                    "user": holder,
                    "chainId": chain_id,
                }),
            )
            .await?;

        let Some(user) = response.user_by_address else {
            return Ok(Vec::new());
        };

        let mut positions = Vec::new();

        for position in user.vault_v2_positions {
            let value = raw_to_float(&position.assets, position.vault.asset.decimals);
            if value == 0.0 {
                continue;
            }
            let apr = if position.vault.avg_net_apy != 0.0 {
                position.vault.avg_net_apy
            } else {
                position.vault.apy
            };
            let earned_7d = value * apr * 7.0 / 365.0;
            let earned_7d_usd = position.assets_usd * apr * 7.0 / 365.0;

            positions.push(DefiPosition {
                protocol: "Morpho".to_string(),
                chain_id,
                chain_name: chain_name.clone(),
                name: position.vault.name,
                underlying_symbol: position.vault.asset.symbol,
                value,
                value_usd: position.assets_usd,
                tvl_usd: 0.0,
                apr,
                earned_7d,
                earned_7d_usd,
            });
        }

        for position in user.vault_positions {
            let value = raw_to_float(&position.state.assets, position.vault.asset.decimals);
            if value == 0.0 {
                continue;
            }
            let apr = if position.vault.state.net_apy != 0.0 {
                position.vault.state.net_apy
            } else {
                position.vault.state.apy
            };
            let earned_7d = value * apr * 7.0 / 365.0;
            let earned_7d_usd = position.state.assets_usd * apr * 7.0 / 365.0;

            positions.push(DefiPosition {
                protocol: "Morpho".to_string(),
                chain_id,
                chain_name: chain_name.clone(),
                name: position.vault.name,
                underlying_symbol: position.vault.asset.symbol,
                value,
                value_usd: position.state.assets_usd,
                tvl_usd: 0.0,
                apr,
                earned_7d,
                earned_7d_usd,
            });
        }

        Ok(positions)
    }

    async fn post_graphql<T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<T> {
        #[derive(Deserialize)]
        struct GraphqlEnvelope<T> {
            data: Option<T>,
            errors: Option<Vec<GraphqlError>>,
        }

        #[derive(Deserialize)]
        struct GraphqlError {
            message: String,
        }

        let response = self
            .client
            .post(endpoint)
            .json(&json!({
                "query": query,
                "variables": variables,
            }))
            .send()
            .await
            .with_context(|| format!("POST {endpoint} failed"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("POST {endpoint} returned {status}: {body}");
        }

        let envelope: GraphqlEnvelope<T> = response.json().await?;
        if let Some(errors) = envelope.errors {
            if !errors.is_empty() {
                anyhow::bail!(
                    "{}",
                    errors
                        .into_iter()
                        .map(|error| error.message)
                        .collect::<Vec<_>>()
                        .join("; ")
                );
            }
        }

        envelope.data.context("GraphQL response missing data")
    }
}

const AAVE_MARKETS_QUERY: &str = r#"
query($chainIds: [ChainId!]!) {
  markets(request: { chainIds: $chainIds }) {
    address
    chain { chainId }
  }
}
"#;

const AAVE_USER_SUPPLIES_QUERY: &str = r#"
query($markets: [MarketInput!]!, $user: EvmAddress!) {
  userSupplies(request: { markets: $markets, user: $user, collateralsOnly: false }) {
    market { chain { chainId name } }
    currency { symbol decimals address }
    balance { amount { value } usd }
    apy { value }
  }
}
"#;

const MORPHO_POSITIONS_QUERY: &str = r#"
query($user: String!, $chainId: Int!) {
  userByAddress(address: $user, chainId: $chainId) {
    vaultV2Positions {
      vault { name apy avgNetApy asset { symbol decimals address } }
      assets assetsUsd
    }
    vaultPositions {
      vault {
        name
        asset { symbol decimals address }
        state { apy netApy }
      }
      state { assets assetsUsd }
    }
  }
}
"#;

#[derive(Deserialize)]
struct AaveMarketsResponse {
    markets: Vec<AaveMarketResponse>,
}

#[derive(Deserialize)]
struct AaveMarketResponse {
    address: String,
    chain: AaveChain,
}

struct AaveMarket {
    address: String,
    chain_id: u64,
}

#[derive(Deserialize)]
struct AaveChain {
    #[serde(rename = "chainId")]
    chain_id: u64,
    #[serde(default)]
    name: String,
}

#[derive(Deserialize)]
struct AaveSuppliesResponse {
    #[serde(rename = "userSupplies")]
    user_supplies: Vec<AaveSupply>,
}

#[derive(Deserialize)]
struct AaveSupply {
    market: AaveSupplyMarket,
    currency: AaveCurrency,
    balance: AaveBalance,
    apy: AaveValue,
}

#[derive(Deserialize)]
struct AaveSupplyMarket {
    chain: AaveChain,
}

#[derive(Deserialize)]
struct AaveCurrency {
    symbol: String,
    #[allow(dead_code)]
    decimals: u8,
    #[allow(dead_code)]
    address: String,
}

#[derive(Deserialize)]
struct AaveBalance {
    amount: AaveValue,
    usd: String,
}

#[derive(Deserialize)]
struct AaveValue {
    value: String,
}

#[derive(Deserialize)]
struct MorphoSnapshotResponse {
    #[serde(rename = "userByAddress")]
    user_by_address: Option<MorphoUser>,
}

#[derive(Deserialize)]
struct MorphoUser {
    #[serde(rename = "vaultV2Positions")]
    vault_v2_positions: Vec<MorphoV2Position>,
    #[serde(rename = "vaultPositions")]
    vault_positions: Vec<MorphoV1Position>,
}

#[derive(Deserialize)]
struct MorphoV2Position {
    vault: MorphoV2Vault,
    assets: BigIntString,
    #[serde(rename = "assetsUsd")]
    assets_usd: f64,
}

#[derive(Deserialize)]
struct MorphoV1Position {
    vault: MorphoV1Vault,
    state: MorphoV1PositionState,
}

#[derive(Deserialize)]
struct MorphoV2Vault {
    name: String,
    apy: f64,
    #[serde(rename = "avgNetApy")]
    avg_net_apy: f64,
    asset: MorphoAsset,
}

#[derive(Deserialize)]
struct MorphoV1Vault {
    name: String,
    asset: MorphoAsset,
    state: MorphoV1VaultState,
}

#[derive(Deserialize)]
struct MorphoV1VaultState {
    apy: f64,
    #[serde(rename = "netApy")]
    net_apy: f64,
}

#[derive(Deserialize)]
struct MorphoV1PositionState {
    assets: BigIntString,
    #[serde(rename = "assetsUsd")]
    assets_usd: f64,
}

#[derive(Deserialize)]
struct MorphoAsset {
    symbol: String,
    decimals: u8,
    #[allow(dead_code)]
    address: String,
}

#[derive(Clone, Debug, Serialize)]
struct BigIntString(String);

impl<'de> Deserialize<'de> for BigIntString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::String(value) => Ok(Self(value)),
            serde_json::Value::Number(value) => Ok(Self(value.to_string())),
            serde_json::Value::Null => Ok(Self(String::new())),
            _ => Err(serde::de::Error::custom("invalid bigint value")),
        }
    }
}

fn parse_f64(value: &str) -> f64 {
    value.parse::<f64>().unwrap_or(0.0)
}

fn raw_to_float(raw: &BigIntString, decimals: u8) -> f64 {
    if raw.0.is_empty() {
        return 0.0;
    }
    parse_f64(&raw.0) / 10f64.powi(decimals as i32)
}
