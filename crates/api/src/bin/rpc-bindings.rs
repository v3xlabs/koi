use std::{fmt::Write, fs, path::PathBuf};

use koi::rpc::{MethodRecord, RPC_METHODS, bindings::DeclarationCollector};
use koi_api::rpc::{
    RpcErrorData, RpcErrorEnvelope, RpcErrorKind, RpcErrorObject, RpcIdentity, RpcRequestEnvelope,
    RpcResponseEnvelope, RpcSuccessEnvelope,
};
use ts_rs::Config;

const GENERATED_HEADER: &str = "/* Generated from the Rust RPC contract. Do not edit. */\n\n";

fn lower_camel(value: &str) -> String {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };

    first.to_lowercase().chain(characters).collect()
}

fn push_aliases(output: &mut String, record: &MethodRecord, config: &Config) {
    writeln!(
        output,
        "export type {}RpcParams = {};",
        record.marker,
        (record.params_ts_name)(config)
    )
    .unwrap();
    writeln!(
        output,
        "export type {}RpcResult = {};",
        record.marker,
        (record.output_ts_name)(config)
    )
    .unwrap();
}

fn push_contract_method(output: &mut String, record: &MethodRecord) {
    writeln!(
        output,
        "    \"{}\": {{ params: RpcBindings.{marker}RpcParams; result: RpcBindings.{marker}RpcResult }};",
        record.name,
        marker = record.marker
    )
    .unwrap();
}

fn push_schema_import(output: &mut String, record: &MethodRecord) {
    writeln!(output, "    {}RpcResultSchema,", lower_camel(record.marker)).unwrap();
}

fn push_wrapper(output: &mut String, record: &MethodRecord) {
    let function = lower_camel(record.marker);
    let schema = format!("{function}RpcResultSchema");

    if (record.takes_params)() {
        writeln!(
            output,
            "    {function}: (params: RpcParams<\"{name}\">): Promise<RpcResult<\"{name}\">> => rpcTransport.call(\"{name}\", params, value => {schema}.parse(value)),",
            name = record.name
        )
        .unwrap();
    } else {
        writeln!(
            output,
            "    {function}: (): Promise<RpcResult<\"{name}\">> => rpcTransport.call(\"{name}\", empty, value => {schema}.parse(value)),",
            name = record.name
        )
        .unwrap();
    }
}

fn dart_type(value: &str) -> String {
    let value = value.trim();
    match value {
        "string" => "String".to_string(),
        "number" => "num".to_string(),
        "boolean" => "bool".to_string(),
        "null" => "void".to_string(),
        _ if value.starts_with("Array<") && value.ends_with('>') => {
            format!("List<{}>", dart_type(&value[6..value.len() - 1]))
        }
        _ => value.to_string(),
    }
}

fn dart_decode(value: &str, expression: &str) -> String {
    let value = value.trim();
    match value {
        "string" => format!("{expression} as String"),
        "number" => format!("{expression} as num"),
        "boolean" => format!("{expression} as bool"),
        "null" => "null".to_string(),
        _ if value.starts_with("Array<") && value.ends_with('>') => {
            let inner = &value[6..value.len() - 1];
            format!(
                "decodeRpcList({expression}, (value) => {})",
                dart_decode(inner, "value")
            )
        }
        _ => format!("decode{value}({expression})"),
    }
}

fn push_dart_wrapper(output: &mut String, record: &MethodRecord, config: &Config) {
    let function = lower_camel(record.marker);
    let params = (record.params_ts_name)(config);
    let result = (record.output_ts_name)(config);
    let result_type = dart_type(&result);
    let decoder = if result == "null" {
        "(_) {}".to_string()
    } else {
        format!("(value) => {}", dart_decode(&result, "value"))
    };

    if (record.takes_params)() {
        writeln!(
            output,
            "  Future<{result_type}> {function}({params} params) => _call('{}', params.toJson(), {decoder});",
            record.name
        )
        .unwrap();
    } else {
        writeln!(
            output,
            "  Future<{result_type}> {function}() => _call('{}', const <String, Object?>{{}}, {decoder});",
            record.name
        )
        .unwrap();
    }
}

fn main() {
    let config = Config::default().with_large_int("number");
    let mut records = RPC_METHODS.iter().collect::<Vec<_>>();
    records.sort_by_key(|record| record.name);
    for pair in records.windows(2) {
        assert_ne!(
            pair[0].name, pair[1].name,
            "duplicate RPC method name: {}",
            pair[0].name
        );
    }

    let mut collector = DeclarationCollector::new(&config);
    collector.collect::<RpcErrorKind>();
    collector.collect::<RpcErrorData>();
    collector.collect::<RpcErrorObject>();
    collector.collect::<RpcIdentity>();
    collector.collect::<RpcRequestEnvelope>();
    collector.collect::<RpcSuccessEnvelope>();
    collector.collect::<RpcErrorEnvelope>();
    collector.collect::<RpcResponseEnvelope>();
    for record in &records {
        (record.collect_types)(&mut collector);
    }

    let mut bindings = GENERATED_HEADER.to_string();
    for declaration in collector.into_declarations().into_values() {
        writeln!(bindings, "export {declaration}\n").unwrap();
    }
    for record in &records {
        push_aliases(&mut bindings, record, &config);
    }

    let mut contract = format!(
        "{GENERATED_HEADER}import type * as RpcBindings from \"./bindings.gen\";\n\nexport type {{ RpcErrorEnvelope, RpcIdentity, RpcRequestEnvelope, RpcResponseEnvelope, RpcSuccessEnvelope }} from \"./bindings.gen\";\n\nexport type RpcMethodMap = {{\n"
    );
    for record in &records {
        push_contract_method(&mut contract, record);
    }
    contract.push_str(
        "};\n\nexport type RpcMethodName = keyof RpcMethodMap;\nexport type RpcParams<TMethod extends RpcMethodName> = RpcMethodMap[TMethod][\"params\"];\nexport type RpcResult<TMethod extends RpcMethodName> = RpcMethodMap[TMethod][\"result\"];\n",
    );

    let mut wrappers = format!("{GENERATED_HEADER}import {{\n");
    for record in &records {
        push_schema_import(&mut wrappers, record);
    }
    wrappers.push_str(
        "} from \"./bindings.zod.gen\";\nimport { createRpcClient } from \"./rpc-transport\";\nimport type { RpcMethodName, RpcParams, RpcResult } from \"./rpc-contract.gen\";\n\nexport const rpcTransport = createRpcClient();\nconst empty = {};\n\nexport const rpc = {\n",
    );
    for record in &records {
        push_wrapper(&mut wrappers, record);
    }
    wrappers.push_str("};\n");

    let mut dart_wrappers = "// Generated from the Rust RPC contract. Do not edit.\n// ignore_for_file: curly_braces_in_flow_control_structures\n\nimport 'dart:convert';\n\nimport 'bridge/api.dart';\nimport 'rpc_models.gen.dart';\n\nfinal class RpcException implements Exception {\n  const RpcException(this.code, this.message);\n  final int code;\n  final String message;\n  @override\n  String toString() => 'RPC error $code: $message';\n}\n\nfinal class RpcClient {\n  RpcClient(this._client);\n  final InProcessClient _client;\n  var _nextId = 0;\n\n  Future<T> _call<T>(String method, Map<String, Object?> params, T Function(Object?) decode) async {\n    final id = ++_nextId;\n    final response = await processMessage(client: _client, message: jsonEncode(<String, Object?>{'jsonrpc': '2.0', 'id': id, 'method': method, 'params': params}));\n    if (response == null) throw StateError('$method returned no response');\n    final envelope = jsonDecode(response) as Map<String, Object?>;\n    final error = envelope['error'];\n    if (error is Map<String, Object?>) throw RpcException(error['code'] as int, error['message'] as String);\n    return decode(envelope['result']);\n  }\n\n".to_string();
    dart_wrappers = dart_wrappers
        .replace("import 'bridge/api.dart';", "import 'bridge/api.dart' as bridge;")
        .replace("final InProcessClient _client;", "final bridge.InProcessClient _client;")
        .replace("await processMessage(", "await bridge.processMessage(");
    for record in &records {
        push_dart_wrapper(&mut dart_wrappers, record, &config);
    }
    dart_wrappers.push_str("}\n");

    let api_directory =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../interfaces/web/src/api");
    fs::write(api_directory.join("bindings.gen.ts"), bindings).unwrap();
    fs::write(api_directory.join("rpc-contract.gen.ts"), contract).unwrap();
    fs::write(api_directory.join("rpc.gen.ts"), wrappers).unwrap();

    let mobile_directory =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../interfaces/mobile/lib/src/core");
    fs::write(mobile_directory.join("rpc.gen.dart"), dart_wrappers).unwrap();
}
