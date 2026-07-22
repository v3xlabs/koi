use alloy::{
    primitives::{Address, B256, address, keccak256},
    providers::DynProvider,
    sol,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    error::KoiError, models::network::identity::NetworkIdentity, rpc::RpcHandler, rpc_method,
    state::AppState,
};

const ENS_REGISTRY: Address = address!("0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e");
const ENS_NETWORK: NetworkIdentity = NetworkIdentity(1);

sol! {
    #[sol(rpc)]
    contract EnsRegistry {
        function resolver(bytes32 node) public view returns (address);
    }

    #[sol(rpc)]
    contract EnsResolver {
        function addr(bytes32 node) public view returns (address);
        function name(bytes32 node) public view returns (string);
    }
}

fn namehash(name: &str) -> B256 {
    let mut node = B256::ZERO;
    for label in name.split('.').rev() {
        if label.is_empty() {
            continue;
        }
        let label_hash = keccak256(label.as_bytes());
        let mut combined = [0u8; 64];
        combined[..32].copy_from_slice(node.as_slice());
        combined[32..].copy_from_slice(label_hash.as_slice());
        node = keccak256(combined);
    }
    node
}

async fn mainnet_provider(state: &AppState) -> Result<DynProvider, KoiError> {
    let rpc = state
        .networks
        .get_pool(&ENS_NETWORK)
        .get_first_rpc(state)
        .await?;
    rpc.get_provider().ok_or(KoiError::Unavailable(
        "no mainnet RPC available".to_string(),
    ))
}

async fn resolver_for(provider: &DynProvider, node: B256) -> Result<Option<Address>, KoiError> {
    let resolver = EnsRegistry::new(ENS_REGISTRY, provider)
        .resolver(node)
        .call()
        .await
        .map_err(|error| KoiError::Unavailable(format!("ENS registry lookup failed: {error}")))?;
    Ok((!resolver.is_zero()).then_some(resolver))
}

async fn forward_resolve(provider: &DynProvider, name: &str) -> Result<Option<Address>, KoiError> {
    let node = namehash(name);
    let Some(resolver) = resolver_for(provider, node).await? else {
        return Ok(None);
    };
    let resolved = EnsResolver::new(resolver, provider)
        .addr(node)
        .call()
        .await
        .map_err(|error| KoiError::Unavailable(format!("ENS resolver lookup failed: {error}")))?;
    Ok((!resolved.is_zero()).then_some(resolved))
}

pub async fn resolve_name(state: &AppState, name: &str) -> Result<Option<String>, KoiError> {
    let name = name.trim().to_lowercase();
    if name.is_empty() || !name.contains('.') {
        return Ok(None);
    }

    let provider = mainnet_provider(state).await?;
    Ok(forward_resolve(&provider, &name)
        .await?
        .map(|address| address.to_checksum(None)))
}

/// Resolves an address to its primary ENS name, forward-verifying the claim
/// as the ENS spec requires.
pub async fn reverse_name(state: &AppState, address: &str) -> Result<Option<String>, KoiError> {
    let address: Address = address
        .trim()
        .parse()
        .map_err(|_| KoiError::InvalidInput("invalid address".to_string()))?;

    let provider = mainnet_provider(state).await?;
    let reverse_node = namehash(&format!("{:x}.addr.reverse", address));
    let Some(resolver) = resolver_for(&provider, reverse_node).await? else {
        return Ok(None);
    };

    let name = EnsResolver::new(resolver, &provider)
        .name(reverse_node)
        .call()
        .await
        .map_err(|error| KoiError::Unavailable(format!("ENS reverse lookup failed: {error}")))?;
    let name = name.trim().to_lowercase();
    if name.is_empty() {
        return Ok(None);
    }

    let verified = forward_resolve(&provider, &name)
        .await?
        .is_some_and(|resolved| resolved == address);
    Ok(verified.then_some(name))
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct EnsResolveParams {
    pub name: String,
}

rpc_method!(EnsResolve, "ens.resolve", EnsResolveParams => Option<String>);

impl RpcHandler for EnsResolve {
    async fn handle(
        state: &AppState,
        params: EnsResolveParams,
    ) -> Result<Option<String>, KoiError> {
        resolve_name(state, &params.name).await
    }
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct EnsReverseParams {
    pub address: String,
}

rpc_method!(EnsReverse, "ens.reverse", EnsReverseParams => Option<String>);

impl RpcHandler for EnsReverse {
    async fn handle(
        state: &AppState,
        params: EnsReverseParams,
    ) -> Result<Option<String>, KoiError> {
        reverse_name(state, &params.address).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_ens_names() {
        assert_eq!(
            namehash("eth").to_string(),
            "0x93cdeb708b7545dc668eb9280176169d1c33cfd8ed6f04690a0bcc88a93fc4ae"
        );
        assert_eq!(
            namehash("foo.eth").to_string(),
            "0xde9b09fd7c5f901e23a3f19fecc54828e9c848539801e86591bd9801b019f84f"
        );
    }

    #[test]
    fn builds_reverse_nodes_from_lowercase_hex() {
        let address: Address = "0x314159265dD8dbb310642f98f50C066173C1259b"
            .parse()
            .unwrap();
        assert_eq!(
            format!("{address:x}.addr.reverse"),
            "314159265dd8dbb310642f98f50c066173c1259b.addr.reverse"
        );
    }
}
