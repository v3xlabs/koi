use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use chrono::Utc;
use serde_json::{Value, json};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::request_params;

#[derive(Clone, Default)]
pub struct Permissions(Arc<RwLock<HashMap<Uuid, HashSet<String>>>>);

impl Permissions {
    pub async fn grant(&self, connection_id: Uuid, methods: &[String]) {
        self.0
            .write()
            .await
            .entry(connection_id)
            .or_default()
            .extend(methods.iter().cloned());
    }

    pub async fn has(&self, connection_id: Uuid, method: &str) -> bool {
        self.0
            .read()
            .await
            .get(&connection_id)
            .is_some_and(|permissions| permissions.contains(method))
    }

    pub async fn list_response(&self, connection_id: Uuid) -> Value {
        let permissions = self
            .0
            .read()
            .await
            .get(&connection_id)
            .cloned()
            .unwrap_or_default();

        json!(
            permissions
                .into_iter()
                .map(|method| json!({
                    "invoker": format!("openlv:{connection_id}"),
                    "parentCapability": method,
                    "caveats": []
                }))
                .collect::<Vec<_>>()
        )
    }
}

pub fn requested_permissions(raw_request: &Value) -> Vec<String> {
    request_params(raw_request)
        .and_then(|params| params.first())
        .or_else(|| raw_request.get("params"))
        .and_then(Value::as_object)
        .map(|request| {
            request
                .keys()
                .filter(|method| grantable_permission(method))
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn grantable_permission(method: &str) -> bool {
    matches!(method, "eth_accounts" | "wallet_getAssets")
}

pub fn requested_permissions_response(permissions: &[String]) -> Value {
    json!(
        permissions
            .iter()
            .map(|method| json!({
                "parentCapability": method,
                "date": Utc::now().timestamp_millis()
            }))
            .collect::<Vec<_>>()
    )
}
