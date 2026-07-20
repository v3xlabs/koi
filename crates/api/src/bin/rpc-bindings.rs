use std::{
    any::{TypeId, type_name},
    collections::{BTreeMap, HashSet},
    fmt::Write,
    fs,
    path::PathBuf,
};

use koi_api::rpc::{
    RpcErrorData, RpcErrorEnvelope, RpcErrorKind, RpcErrorObject, RpcIdentity, RpcMethod,
    RpcRequestEnvelope, RpcResponseEnvelope, RpcSuccessEnvelope, methods::*,
};
use ts_rs::{Config, TS, TypeVisitor};

const GENERATED_HEADER: &str = "/* Generated from the Rust RPC contract. Do not edit. */\n\n";

struct DeclarationCollector<'a> {
    config: &'a Config,
    seen: HashSet<TypeId>,
    declarations: BTreeMap<String, String>,
}

impl<'a> DeclarationCollector<'a> {
    fn new(config: &'a Config) -> Self {
        Self {
            config,
            seen: HashSet::new(),
            declarations: BTreeMap::new(),
        }
    }

    fn collect<T: TS + 'static + ?Sized>(&mut self) {
        self.visit::<T>();
    }

    fn render(self) -> String {
        let mut output = GENERATED_HEADER.to_string();

        for declaration in self.declarations.into_values() {
            writeln!(output, "export {declaration}\n").unwrap();
        }

        output
    }
}

impl TypeVisitor for DeclarationCollector<'_> {
    fn visit<T: TS + 'static + ?Sized>(&mut self) {
        if !self.seen.insert(TypeId::of::<T>()) {
            return;
        }

        if T::output_path().is_some() {
            let declaration = normalize_declaration(T::decl(self.config));
            let previous = self
                .declarations
                .insert(T::ident(self.config), declaration.clone());

            assert!(
                previous.as_ref().is_none_or(|value| value == &declaration),
                "conflicting TypeScript declarations for {}",
                T::ident(self.config)
            );
        }

        T::visit_dependencies(self);
        T::visit_generics(self);
    }
}

fn normalize_declaration(declaration: String) -> String {
    let mut declaration = declaration.replace("Record<symbol, never>", "Record<string, never>");
    let prefix = "{ [key in string]: ";

    while let Some(start) = declaration.find(prefix) {
        let value_start = start + prefix.len();
        let end = declaration[value_start..]
            .find(" }")
            .map(|offset| value_start + offset)
            .expect("ts-rs string map declaration should have a closing brace");
        let value = declaration[value_start..end].to_string();

        declaration.replace_range(start..end + 2, &format!("Record<string, {value}>"));
    }

    declaration
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
}

fn marker_name<M: RpcMethod>() -> &'static str {
    type_name::<M>().rsplit("::").next().unwrap()
}

fn lower_camel(value: &str) -> String {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };

    first.to_lowercase().chain(characters).collect()
}

fn push_aliases<M: RpcMethod>(output: &mut String, config: &Config) {
    let marker = marker_name::<M>();

    writeln!(
        output,
        "export type {marker}RpcParams = {};",
        M::Params::name(config)
    )
    .unwrap();
    writeln!(
        output,
        "export type {marker}RpcResult = {};",
        M::Output::name(config)
    )
    .unwrap();
}

fn push_contract_method<M: RpcMethod>(output: &mut String) {
    let marker = marker_name::<M>();

    writeln!(
        output,
        "    \"{}\": {{ params: RpcBindings.{marker}RpcParams; result: RpcBindings.{marker}RpcResult }};",
        M::NAME
    )
    .unwrap();
}

fn push_schema_import<M: RpcMethod>(output: &mut String) {
    writeln!(
        output,
        "    {}RpcResultSchema,",
        lower_camel(marker_name::<M>())
    )
    .unwrap();
}

fn push_wrapper<M: RpcMethod>(output: &mut String) {
    let marker = marker_name::<M>();
    let function = lower_camel(marker);
    let schema = format!("{function}RpcResultSchema");

    if TypeId::of::<M::Params>() == TypeId::of::<koi_api::rpc::EmptyParams>() {
        writeln!(
            output,
            "    {function}: (): Promise<RpcResult<\"{}\">> => rpcTransport.call(\"{}\", empty, value => {schema}.parse(value)),",
            M::NAME,
            M::NAME
        )
        .unwrap();
    } else {
        writeln!(
            output,
            "    {function}: (params: RpcParams<\"{}\">): Promise<RpcResult<\"{}\">> => rpcTransport.call(\"{}\", params, value => {schema}.parse(value)),",
            M::NAME,
            M::NAME,
            M::NAME
        )
        .unwrap();
    }
}

macro_rules! for_each_method {
    ($( $marker:ident, $params:ty, $output:ty, $name:literal; )*) => {
        fn collect_method_declarations(collector: &mut DeclarationCollector<'_>) {
            $(
                collector.collect::<<$marker as RpcMethod>::Params>();
                collector.collect::<<$marker as RpcMethod>::Output>();
            )*
        }

        fn push_method_aliases(output: &mut String, config: &Config) {
            $(push_aliases::<$marker>(output, config);)*
        }

        fn push_contract_methods(output: &mut String) {
            $(push_contract_method::<$marker>(output);)*
        }

        fn push_schema_imports(output: &mut String) {
            $(push_schema_import::<$marker>(output);)*
        }

        fn push_wrappers(output: &mut String) {
            $(push_wrapper::<$marker>(output);)*
        }
    };
}

koi_api::rpc_method_registry!(for_each_method);

fn main() {
    let config = Config::default().with_large_int("number");
    let mut collector = DeclarationCollector::new(&config);

    collector.collect::<RpcErrorKind>();
    collector.collect::<RpcErrorData>();
    collector.collect::<RpcErrorObject>();
    collector.collect::<RpcIdentity>();
    collector.collect::<RpcRequestEnvelope>();
    collector.collect::<RpcSuccessEnvelope>();
    collector.collect::<RpcErrorEnvelope>();
    collector.collect::<RpcResponseEnvelope>();
    collect_method_declarations(&mut collector);

    let mut bindings = collector.render();
    push_method_aliases(&mut bindings, &config);

    let mut contract = format!(
        "{GENERATED_HEADER}import type * as RpcBindings from \"./bindings.gen\";\n\nexport type {{ RpcErrorEnvelope, RpcIdentity, RpcRequestEnvelope, RpcResponseEnvelope, RpcSuccessEnvelope }} from \"./bindings.gen\";\n\nexport type RpcMethodMap = {{\n"
    );
    push_contract_methods(&mut contract);
    contract.push_str(
        "};\n\nexport type RpcMethodName = keyof RpcMethodMap;\nexport type RpcParams<TMethod extends RpcMethodName> = RpcMethodMap[TMethod][\"params\"];\nexport type RpcResult<TMethod extends RpcMethodName> = RpcMethodMap[TMethod][\"result\"];\n",
    );

    let mut wrappers = format!("{GENERATED_HEADER}import {{\n");
    push_schema_imports(&mut wrappers);
    wrappers.push_str(
        "} from \"./bindings.zod.gen\";\nimport { createRpcClient } from \"./rpc-transport\";\nimport type { RpcMethodName, RpcParams, RpcResult } from \"./rpc-contract.gen\";\n\nexport const rpcTransport = createRpcClient();\nconst empty = {};\n\nexport const rpc = {\n",
    );
    push_wrappers(&mut wrappers);
    wrappers.push_str("};\n");

    let api_directory =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../interfaces/web/src/api");
    fs::write(api_directory.join("bindings.gen.ts"), bindings).unwrap();
    fs::write(api_directory.join("rpc-contract.gen.ts"), contract).unwrap();
    fs::write(api_directory.join("rpc.gen.ts"), wrappers).unwrap();
}
