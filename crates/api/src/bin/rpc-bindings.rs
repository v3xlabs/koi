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

    let api_directory =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../interfaces/web/src/api");
    fs::write(api_directory.join("bindings.gen.ts"), bindings).unwrap();
    fs::write(api_directory.join("rpc-contract.gen.ts"), contract).unwrap();
    fs::write(api_directory.join("rpc.gen.ts"), wrappers).unwrap();
}
