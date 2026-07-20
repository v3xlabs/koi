use std::{collections::HashMap, sync::Arc};

use anyhow::{Context, Result};
use futures_util::{Sink, SinkExt, Stream, StreamExt, lock::Mutex};
use koi::models::{
    account::{Account, balances::AccountBalances},
    asset::{Asset, metadata::AssetMetadataDiscovery},
    network::{Network, endpoint::NetworkEndpoint, pool::RpcPoolStats},
    quoter::Quoter,
    tx::Tx,
    vendor::flags::{VendorFlag, VendorFlagInfo},
};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        Error as WebSocketError, Message,
        client::IntoClientRequest,
        http::{Request, header::ORIGIN},
    },
};

const DISPLAY_CURRENCY: &str = "fiat:usd";

#[derive(Clone)]
pub struct ApiClient {
    base: String,
    socket: Arc<Mutex<Option<Box<dyn RpcSocket>>>>,
}

trait RpcSocket:
    Sink<Message, Error = WebSocketError>
    + Stream<Item = Result<Message, WebSocketError>>
    + Send
    + Unpin
{
}

impl<T> RpcSocket for T where
    T: Sink<Message, Error = WebSocketError>
        + Stream<Item = Result<Message, WebSocketError>>
        + Send
        + Unpin
{
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base: base_url.trim_end_matches('/').to_string(),
            socket: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn health(&self) -> Result<()> {
        let _: String = self.call("system.ping", json!({})).await?;
        Ok(())
    }

    pub async fn accounts(&self) -> Result<Vec<Account>> {
        self.call("account.list", json!({})).await
    }

    pub async fn account_balances(
        &self,
        account_identity: u64,
        fresh: bool,
    ) -> Result<AccountBalances> {
        self.call(
            "account.balance.list",
            json!({
                "account_identity": account_identity,
                "display_currency": DISPLAY_CURRENCY,
                "fresh": fresh,
            }),
        )
        .await
    }

    pub async fn account_transactions(&self, account_identity: u64) -> Result<Vec<Tx>> {
        self.call(
            "account.transaction.list",
            json!({ "account_identity": account_identity }),
        )
        .await
    }

    pub async fn networks(&self) -> Result<Vec<Network>> {
        self.call("network.list", json!({})).await
    }

    pub async fn network_rpc_stats(&self, network_identity: u64) -> Result<RpcPoolStats> {
        self.call(
            "network.rpcStats",
            json!({ "network_identity": network_identity }),
        )
        .await
    }

    pub async fn network_endpoints(&self, network_identity: u64) -> Result<Vec<NetworkEndpoint>> {
        self.call(
            "network.endpoint.list",
            json!({ "network_identity": network_identity }),
        )
        .await
    }

    pub async fn delete_network_endpoint(
        &self,
        network_identity: u64,
        endpoint_identity: i32,
    ) -> Result<()> {
        self.call(
            "network.endpoint.delete",
            json!({
                "network_identity": network_identity,
                "endpoint_identity": endpoint_identity,
            }),
        )
        .await
    }

    pub async fn network_endpoint_next_id(&self, network_identity: u64) -> Result<i32> {
        self.call(
            "network.endpoint.nextIdentity",
            json!({ "network_identity": network_identity }),
        )
        .await
    }

    pub async fn create_network(&self, network: &Network) -> Result<Network> {
        self.call("network.create", json!({ "input": network }))
            .await
    }

    pub async fn create_network_endpoint(
        &self,
        network_identity: u64,
        endpoint: &NetworkEndpoint,
    ) -> Result<NetworkEndpoint> {
        self.call(
            "network.endpoint.create",
            json!({ "network_identity": network_identity, "input": endpoint }),
        )
        .await
    }

    pub async fn assets(&self) -> Result<HashMap<String, Asset>> {
        let assets: Vec<Asset> = self.call("asset.list", json!({})).await?;
        Ok(assets
            .into_iter()
            .map(|asset| (asset.asset_identity.to_string(), asset))
            .collect())
    }

    pub async fn delete_asset(&self, asset_identity: &str) -> Result<()> {
        self.call("asset.delete", json!({ "asset_identity": asset_identity }))
            .await
    }

    pub async fn create_asset(&self, asset: &Asset) -> Result<Asset> {
        self.call("asset.create", json!({ "input": asset })).await
    }

    pub async fn asset_metadata_discovery(
        &self,
        asset_identity: &str,
    ) -> Result<AssetMetadataDiscovery> {
        self.call(
            "asset.discoverMetadata",
            json!({ "asset_identity": asset_identity }),
        )
        .await
    }

    pub async fn quoters(&self) -> Result<Vec<Quoter>> {
        self.call("quoter.list", json!({})).await
    }

    pub async fn vendors(&self) -> Result<Vec<VendorFlag>> {
        self.call("vendor.listEnabled", json!({})).await
    }

    pub async fn all_vendors(&self) -> Result<Vec<VendorFlagInfo>> {
        self.call("vendor.listAll", json!({})).await
    }

    pub async fn set_vendor(&self, flag: &str, enabled: bool) -> Result<()> {
        self.call(
            if enabled {
                "vendor.enable"
            } else {
                "vendor.disable"
            },
            json!({ "flag": flag }),
        )
        .await
    }

    async fn call<T: DeserializeOwned>(&self, method: &str, params: Value) -> Result<T> {
        let mut socket = self.socket.lock().await;
        if socket.is_none() {
            *socket = Some(self.connect().await?);
        }
        let Some(mut active_socket) = socket.take() else {
            anyhow::bail!("Koi RPC connection was not initialized")
        };

        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });
        if let Err(error) = active_socket
            .send(Message::Text(request.to_string().into()))
            .await
        {
            return Err(error)
                .with_context(|| format!("could not send Koi RPC request for {method}"));
        }

        while let Some(message) = active_socket.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    *socket = Some(active_socket);
                    return decode_response(&text, method);
                }
                Ok(Message::Close(frame)) => {
                    anyhow::bail!("Koi RPC connection closed: {frame:?}");
                }
                Err(error) => {
                    return Err(error).context("could not read Koi RPC response");
                }
                _ => {}
            }
        }

        anyhow::bail!("Koi RPC connection closed before responding to {method}")
    }

    async fn connect(&self) -> Result<Box<dyn RpcSocket>> {
        let request = self.websocket_request()?;
        let (socket, _) = connect_async(request)
            .await
            .with_context(|| format!("could not connect to Koi RPC at {}", self.base))?;
        Ok(Box::new(socket))
    }

    fn websocket_request(&self) -> Result<Request<()>> {
        let uri = self
            .base
            .parse::<tokio_tungstenite::tungstenite::http::Uri>()
            .with_context(|| format!("invalid Koi server URL: {}", self.base))?;
        let scheme = uri
            .scheme_str()
            .context("Koi server URL must include http or https")?;
        let websocket_scheme = match scheme {
            "http" => "ws",
            "https" => "wss",
            _ => anyhow::bail!("Koi server URL must use http or https"),
        };
        let authority = uri
            .authority()
            .context("Koi server URL must include a host")?;
        let origin = format!("{scheme}://{authority}");
        let websocket_url = format!("{websocket_scheme}://{authority}/rpc");

        let mut request = websocket_url
            .into_client_request()
            .context("could not build Koi RPC request")?;
        let origin = origin
            .parse()
            .context("could not encode Koi RPC origin header")?;
        request.headers_mut().insert(ORIGIN, origin);
        Ok(request)
    }
}

fn decode_response<T: DeserializeOwned>(response: &str, method: &str) -> Result<T> {
    let response: Value = serde_json::from_str(response)
        .with_context(|| format!("Koi RPC returned invalid JSON for {method}"))?;

    if let Some(error) = response.get("error") {
        let message = error
            .get("data")
            .and_then(|data| data.get("message"))
            .or_else(|| error.get("message"))
            .and_then(Value::as_str)
            .unwrap_or("unknown Koi RPC error");
        anyhow::bail!("Koi RPC {method} failed: {message}")
    }

    let result = response
        .get("result")
        .context("Koi RPC response did not contain a result")?;
    serde_json::from_value(result.clone())
        .with_context(|| format!("could not decode Koi RPC result for {method}"))
}
