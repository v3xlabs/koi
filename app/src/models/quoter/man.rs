use std::{collections::HashMap, sync::Mutex};

use alloy::{
    primitives::U256,
    providers::{DynProvider, Provider},
};
use sqlx::{SqlitePool, query_as};
use tracing::{info, warn};

use super::Quoter;
use crate::{
    error::KoiError,
    models::{
        asset::{Asset, identity::AssetIdentity},
        network::{Network, identity::NetworkIdentity},
    },
    state::{AppState, DB},
};

use eth_prices::{
    asset::AssetIdentifier,
    network::Network as EthPricesNetwork,
    quoter::AnyQuoter,
    router::{Router, route::Route},
};

pub struct QuoterManager {
    pub quoters: Mutex<HashMap<String, Quoter>>,
    pub routes:
        Mutex<HashMap<NetworkIdentity, HashMap<AssetIdentity, HashMap<AssetIdentity, Route>>>>,
    pub graph: Mutex<HashMap<NetworkIdentity, Router>>,
}

pub struct QuoteInput {
    pub asset_in: AssetIdentity,
    pub asset_out: AssetIdentity,
    pub amount: U256,
}

pub struct QuoteOutput {
    pub asset_in: AssetIdentity,
    pub asset_out: AssetIdentity,
    pub amount: U256,
    pub price: U256,
}

impl QuoterManager {
    pub async fn init(database: &SqlitePool) -> Result<Self, KoiError> {
        let quoters = query_as::<_, Quoter>("SELECT * FROM quoters")
            .fetch_all(database)
            .await?;

        let me = Self {
            quoters: Mutex::new(
                quoters
                    .into_iter()
                    .map(|quoter| (quoter.quoter_identity.clone(), quoter))
                    .collect(),
            ),
            routes: Mutex::new(HashMap::new()),
            graph: Mutex::new(HashMap::new()),
        };

        me.build_graph(database).await?;

        Ok(me)
    }

    pub async fn build_graph(&self, database: &DB) -> Result<(), KoiError> {
        let networks = Network::all(database).await?;

        for network in networks {
            self.build_network_graph(database, &network.network_identity)
                .await?;
        }

        Ok(())
    }

    async fn build_network_graph(
        &self,
        database: &DB,
        network_identity: &NetworkIdentity,
    ) -> Result<(), KoiError> {
        info!("Building graph for network {}", network_identity);

        // TODO: respect "enabled" flag
        let quoters = Quoter::get_by_network_id(database, network_identity).await?;

        let quoters: Vec<AnyQuoter> = quoters.iter().map(|x| x.into()).collect();

        let graph = Router::from_iter(quoters);

        info!("Graph {}", graph.to_graphviz());

        let base_asset: AssetIdentity = AssetIdentity::Fiat("usd".to_string());
        let base_token: AssetIdentifier = base_asset.clone().into();

        let assets = Asset::get_by_network_id(database, network_identity).await?;

        info!("Pre-computing routes for {} assets", assets.len());

        let mut map: HashMap<AssetIdentity, HashMap<AssetIdentity, Route>> = HashMap::new();

        for asset in assets {
            let token: AssetIdentifier = asset.asset_identity.clone().into();
            let route = graph.compute(&token, &base_token);
            match route {
                Ok(route) => {
                    map.insert(
                        asset.asset_identity.clone(),
                        HashMap::from([(base_asset.clone(), route)]),
                    );
                }
                Err(e) => {
                    warn!(
                        "Error pre-computing route for asset {}: {}",
                        asset.asset_identity, e
                    );
                }
            }
        }

        self.routes
            .lock()
            .unwrap()
            .insert(network_identity.clone(), map);

        self.graph
            .lock()
            .unwrap()
            .insert(network_identity.clone(), graph);

        Ok(())
    }

    fn route(
        &self,
        network_identity: &NetworkIdentity,
        asset_in: &AssetIdentity,
        asset_out: &AssetIdentity,
    ) -> Result<Route, KoiError> {
        if let Some(route) = self
            .routes
            .lock()
            .expect("routes mutex poisoned")
            .get(network_identity)
            .and_then(|routes| routes.get(asset_in))
            .and_then(|routes| routes.get(asset_out))
            .cloned()
        {
            return Ok(route);
        }

        let graph = self
            .graph
            .lock()
            .expect("graph mutex poisoned")
            .get(network_identity)
            .cloned()
            .ok_or(KoiError::Internal("Graph not found".to_string()))?;

        let token_in: AssetIdentifier = asset_in.clone().into();
        let token_out: AssetIdentifier = asset_out.clone().into();
        let route = graph.compute(&token_in, &token_out).map_err(|e| {
            KoiError::Internal(format!(
                "Error computing route from asset {} to {}: {}",
                asset_in, asset_out, e
            ))
        })?;

        let route = self
            .routes
            .lock()
            .expect("routes mutex poisoned")
            .entry(network_identity.clone())
            .or_default()
            .entry(asset_in.clone())
            .or_default()
            .entry(asset_out.clone())
            .or_insert(route)
            .clone();

        Ok(route)
    }

    pub async fn quote(
        &self,
        state: &AppState,
        network_identity: &NetworkIdentity,
        asset_in: &AssetIdentity,
        asset_out: &AssetIdentity,
        amount_in: U256,
    ) -> Result<U256, KoiError> {
        let route = self.route(network_identity, asset_in, asset_out)?;
        let provider = state
            .networks
            .get_pool(network_identity)
            .get_first_rpc(state)
            .await?;
        let rpc = provider
            .get_provider()
            .ok_or(KoiError::Internal("No provider found for RPC".to_string()))?;

        let block = rpc
            .get_block_number()
            .await
            .map_err(|_| KoiError::Internal("Failed to get block number".to_string()))?;

        let network = EthPricesNetwork::EVM(network_identity.0, block, rpc);

        route
            .quote(&network, amount_in)
            .await
            .map_err(KoiError::from)
    }

    pub async fn quote_b(
        &self,
        rpc: &DynProvider,
        block: u64,
        input: &AssetIdentity,
        output: &AssetIdentity,
        amount: U256,
    ) -> Result<U256, KoiError> {
        let network_identity = input
            .unwrap_network()
            .ok_or(KoiError::Internal("Input asset has no network".to_string()))?;
        let route = self.route(&network_identity, input, output)?;
        let network = EthPricesNetwork::EVM(network_identity.0, block, rpc.clone());
        route.quote(&network, amount).await.map_err(KoiError::from)
    }
}
