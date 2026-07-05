use std::{collections::HashMap, sync::Mutex, time::Duration};

use alloy::{
    primitives::U256,
    providers::{DynProvider, Provider},
};
use moka::future::Cache;
use sqlx::{SqlitePool, query_as};
use tracing::{info, warn};

use super::Quoter;
use crate::{
    error::KoiError,
    models::{
        asset::{Asset, identity::AssetIdentity},
        network::{Network, identity::NetworkIdentity},
        vendor::{flags::VendorFlag, man::VendorManager},
    },
    state::{AppState, DB},
};

use eth_prices::{
    asset::AssetIdentifier,
    network::{NetworkId, NetworkTime},
    quoter::AnyQuoter,
    router::{Router, route::Route},
};

#[derive(Hash, Eq, PartialEq)]
pub struct QuoteCacheKey {
    pub network_identity: NetworkIdentity,
    pub block: u64,
    pub input: AssetIdentity,
    pub amount: U256,
}

pub struct QuoterManager {
    pub quoters: Mutex<HashMap<String, Quoter>>,
    pub routes:
        Mutex<HashMap<NetworkIdentity, HashMap<AssetIdentity, HashMap<AssetIdentity, Route>>>>,
    pub graph: Mutex<HashMap<NetworkIdentity, Router>>,

    pub cache: Cache<QuoteCacheKey, U256>,
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
    pub async fn init(database: &SqlitePool, vendors: &VendorManager) -> Result<Self, KoiError> {
        let quoters = query_as::<_, Quoter>("SELECT * FROM quoters")
            .fetch_all(database)
            .await?;

        let cache = Cache::builder()
            .max_capacity(10_000)
            .time_to_live(Duration::from_secs(60 * 10))
            .time_to_idle(Duration::from_secs(60 * 3))
            .build();

        let me = Self {
            quoters: Mutex::new(
                quoters
                    .into_iter()
                    .map(|quoter| (quoter.quoter_identity.clone(), quoter))
                    .collect(),
            ),
            routes: Mutex::new(HashMap::new()),
            graph: Mutex::new(HashMap::new()),
            cache,
        };

        me.build_graph(database, vendors).await?;

        Ok(me)
    }

    pub async fn build_graph(
        &self,
        database: &DB,
        vendors: &VendorManager,
    ) -> Result<(), KoiError> {
        self.cache.invalidate_all();

        let mut fiat_graph = Router::default();
        if vendors.has_flag(VendorFlag::EcbQuoter) {
            fiat_graph = fiat_graph.with_ecb();
        }
        self.graph
            .lock()
            .expect("graph mutex poisoned")
            .insert(NetworkIdentity(0), fiat_graph);

        let networks = Network::all(database).await?;

        for network in networks {
            self.build_network_graph(database, vendors, &network.network_identity)
                .await?;
        }

        Ok(())
    }

    async fn build_network_graph(
        &self,
        database: &DB,
        vendors: &VendorManager,
        network_identity: &NetworkIdentity,
    ) -> Result<(), KoiError> {
        info!("Building graph for network {}", network_identity);

        // TODO: respect "enabled" flag
        let quoters = Quoter::get_by_network_id(database, network_identity).await?;

        let quoters: Vec<AnyQuoter> = quoters
            .iter()
            .map(|x| x.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        let mut graph = Router::from_iter(quoters);
        if vendors.has_flag(VendorFlag::EcbQuoter) {
            graph = graph.with_ecb();
        }

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
        if network_identity == &NetworkIdentity(0) {
            let network = NetworkTime::with_fiat_now().instant();
            return route
                .quote(&network, amount_in)
                .await
                .map_err(KoiError::from);
        }

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

        let network = NetworkTime::EVM(NetworkId::from(network_identity.0), block, rpc)
            .instant()
            .with_now()
            .unwrap();

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
        amount: U256,
        asset_out: &AssetIdentity,
    ) -> Result<U256, KoiError> {
        if amount == U256::ZERO {
            warn!("Quote amount is zero");
            return Ok(U256::ZERO);
        }

        let network_identity = input
            .unwrap_network()
            .ok_or(KoiError::Internal("Input asset has no network".to_string()))?;

        let key = QuoteCacheKey {
            network_identity: network_identity.clone(),
            block,
            amount,
            input: input.clone(),
        };

        let x = self
            .cache
            .try_get_with(key, async move {
                let route = self.route(&network_identity, input, &asset_out);
                match route {
                    Ok(route) => {
                        let network = NetworkTime::EVM(
                            NetworkId::from(network_identity.0),
                            block,
                            rpc.clone(),
                        )
                        .instant()
                        .with_now()
                        .unwrap();
                        route.quote(&network, amount).await.map_err(KoiError::from)
                    }
                    Err(e) => Err(KoiError::Internal(format!(
                        "Error computing route from asset {} to {}: {}",
                        input, asset_out, e
                    ))),
                }
            })
            .await?;

        Ok(x)
    }
}
