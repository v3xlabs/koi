//! JSON-RPC behavior tests.

use std::sync::Arc;

use koi::{
    config::Configuration,
    db::connect,
    models::{
        abi::AbiManager,
        account::{balance_cache::BalanceCacheManager, rpc::AccountBalancesParams},
        network::manager::NetworkManager,
        quoter::man::QuoterManager,
        vendor::man::VendorManager,
    },
    state::State,
};
use serde_json::{Value, json};
use tempfile::TempDir;

use super::*;

async fn test_dispatcher() -> (TempDir, Dispatcher) {
    let directory = tempfile::tempdir().unwrap();
    let database_path = directory.path().join("rpc.db");
    let database_url = format!("sqlite://{}", database_path.display());
    let database = connect(&database_url, None).await.unwrap();
    let vendors = VendorManager::init(&database).await.unwrap();
    let quoters = QuoterManager::init(&database).await.unwrap();
    let state = Arc::new(State {
        config: Configuration {
            database_url,
            abi_cache_dir: directory.path().join("abis").display().to_string(),
            ..Configuration::default()
        },
        database,
        networks: NetworkManager::default(),
        quoters,
        balances: BalanceCacheManager::new(),
        vendors,
        abis: AbiManager::new(directory.path().join("abis")),
    });

    (directory, Dispatcher::new(state))
}

async fn response(dispatcher: &Dispatcher, request: &str) -> Value {
    serde_json::from_str(&dispatcher.process_message(request).await.unwrap()).unwrap()
}

#[test]
fn protocol_constants_match_limits() {
    assert_eq!(MAX_MESSAGE_BYTES, 8_388_608);
    assert_eq!(MAX_BATCH_ENTRIES, 128);
    assert_eq!(MAX_IN_FLIGHT_CALLS, 128);
}

#[tokio::test]
async fn single_request_and_notification_follow_json_rpc() {
    let (_directory, dispatcher) = test_dispatcher().await;
    let single = response(
        &dispatcher,
        r#"{"jsonrpc":"2.0","id":7,"method":"system.ping","params":{}}"#,
    )
    .await;

    assert_eq!(single, json!({"jsonrpc": "2.0", "id": 7, "result": "OK"}));
    assert_eq!(
        dispatcher
            .process_message(r#"{"jsonrpc":"2.0","method":"system.ping","params":{}}"#)
            .await,
        None
    );
}

#[tokio::test]
async fn typed_in_process_dispatch_uses_the_same_method_markers() {
    let (_directory, dispatcher) = test_dispatcher().await;

    assert_eq!(
        dispatcher
            .call::<SystemPing>(EmptyParams::default())
            .await
            .unwrap(),
        "OK"
    );
}

#[tokio::test]
async fn mixed_batches_are_concurrent_but_response_order_is_stable() {
    let (_directory, dispatcher) = test_dispatcher().await;
    let batch = response(
        &dispatcher,
        r#"[
                {"jsonrpc":"2.0","id":1,"method":"system.ping","params":{}},
                {"jsonrpc":"2.0","method":"system.ping","params":{}},
                {"jsonrpc":"2.0","id":2,"method":"missing","params":{}},
                {"jsonrpc":"2.0","id":3,"method":"system.ping","params":{"extra":true}}
            ]"#,
    )
    .await;

    let entries = batch.as_array().unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0]["id"], 1);
    assert_eq!(entries[0]["result"], "OK");
    assert_eq!(entries[1]["id"], 2);
    assert_eq!(entries[1]["error"]["code"], -32601);
    assert_eq!(entries[2]["id"], 3);
    assert_eq!(entries[2]["error"]["code"], -32602);
}

#[tokio::test]
async fn notification_only_batches_have_no_response() {
    let (_directory, dispatcher) = test_dispatcher().await;

    assert_eq!(
        dispatcher
            .process_message(
                r#"[
                        {"jsonrpc":"2.0","method":"system.ping","params":{}},
                        {"jsonrpc":"2.0","method":"system.ping","params":{}}
                    ]"#,
            )
            .await,
        None
    );
}

#[tokio::test]
async fn protocol_errors_use_standard_codes() {
    let (_directory, dispatcher) = test_dispatcher().await;

    assert_eq!(response(&dispatcher, "{").await["error"]["code"], -32700);
    assert_eq!(response(&dispatcher, "[]").await["error"]["code"], -32600);
    assert_eq!(
        response(
            &dispatcher,
            r#"{"jsonrpc":"2.0","id":1,"method":"system.ping","params":[]}"#,
        )
        .await["error"]["code"],
        -32602
    );

    let oversized_batch = Value::Array(vec![
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "system.ping",
            "params": {}
        });
        MAX_BATCH_ENTRIES + 1
    ]);
    assert_eq!(
        response(&dispatcher, &oversized_batch.to_string()).await["error"]["code"],
        -32600
    );
}

#[tokio::test]
async fn duplicate_ids_remain_correlated_in_request_order() {
    let (_directory, dispatcher) = test_dispatcher().await;
    let batch = response(
        &dispatcher,
        r#"[
                {"jsonrpc":"2.0","id":9,"method":"system.ping","params":{}},
                {"jsonrpc":"2.0","id":9,"method":"account.list","params":{}}
            ]"#,
    )
    .await;
    let entries = batch.as_array().unwrap();

    assert_eq!(
        entries[0],
        json!({"jsonrpc": "2.0", "id": 9, "result": "OK"})
    );
    assert_eq!(entries[1], json!({"jsonrpc": "2.0", "id": 9, "result": []}));
}

#[tokio::test]
async fn application_errors_have_typed_safe_data() {
    let (_directory, dispatcher) = test_dispatcher().await;
    let missing = response(
        &dispatcher,
        r#"{"jsonrpc":"2.0","id":4,"method":"account.get","params":{"account_identity":999}}"#,
    )
    .await;

    assert_eq!(missing["error"]["code"], -32000);
    assert_eq!(missing["error"]["data"]["kind"], "not_found");
    assert_eq!(missing["error"]["data"]["message"], "resource not found");
}

#[tokio::test]
async fn account_crud_returns_direct_domain_values_and_null_units() {
    let (_directory, dispatcher) = test_dispatcher().await;
    let created = response(
            &dispatcher,
            r#"{
                "jsonrpc":"2.0",
                "id":1,
                "method":"account.create",
                "params":{"input":{
                    "name":"Main",
                    "networks":[],
                    "metadata":{"type":"view","evm_address":"0x0000000000000000000000000000000000000000"},
                    "group_id":null,
                    "display_order":0
                }}
            }"#,
        )
        .await;

    assert_eq!(created["result"]["account_identity"], 1);
    assert_eq!(created["result"]["metadata"]["type"], "view");
    assert!(created["result"]["group_id"].is_null());

    let rejected_identity = response(
        &dispatcher,
        r#"{
            "jsonrpc":"2.0",
            "id":2,
            "method":"account.create",
            "params":{"input":{
                "account_identity":2,
                "name":"Other",
                "networks":[],
                "metadata":{"type":"view","evm_address":"0x0000000000000000000000000000000000000000"},
                "display_order":0
            }}
        }"#,
    )
    .await;
    assert_eq!(rejected_identity["error"]["code"], -32602);

    let listed = response(
        &dispatcher,
        r#"{"jsonrpc":"2.0","id":3,"method":"account.list","params":{}}"#,
    )
    .await;
    assert_eq!(listed["result"].as_array().unwrap().len(), 1);

    let layout = response(
        &dispatcher,
        r#"{"jsonrpc":"2.0","id":4,"method":"account.layout.get","params":{}}"#,
    )
    .await;
    assert_eq!(layout["result"]["accounts"][0]["name"], "Main");

    let deleted = response(
        &dispatcher,
        r#"{"jsonrpc":"2.0","id":5,"method":"account.delete","params":{"account_identity":1}}"#,
    )
    .await;
    assert_eq!(deleted["result"], Value::Null);
}

#[test]
fn optional_balance_fresh_flag_defaults_to_false() {
    let params = serde_json::from_value::<AccountBalancesParams>(json!({
        "account_identity": 1,
        "display_currency": "fiat:usd"
    }))
    .unwrap();

    assert_eq!(params.fresh, None);
}
