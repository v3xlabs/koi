use std::{fmt::Write, fs, path::PathBuf};

use koi_api::rpc::{
    AccountAssetBalanceParams, AccountAssetParams, AccountBalancesParams, AccountCreateParams,
    AccountParams, AccountUpdateParams, AssetCreateParams, AssetParams, AssetQuoteParams,
    AssetUpdateParams, DecodeParams, DeriveMnemonicInput, DeriveMnemonicParams,
    DeriveMnemonicResult, DerivePrivateKeyParams, EmptyParams, EndpointCreateParams,
    EndpointParams, EndpointUpdateParams, GroupCreateParams, GroupParams, GroupUpdateParams,
    LayoutUpdateParams, NetworkCreateParams, NetworkParams, NetworkUpdateParams,
    QuoterCreateParams, QuoterDiscoverParams, QuoterParams, QuoterUpdateParams, RpcMethod,
    SimulateParams, VendorParams,
    methods::{
        AccountAssetAdd, AccountAssetBalance, AccountAssetList, AccountAssetRemove,
        AccountBalanceList, AccountCreate, AccountDelete, AccountDerivationDefaultPath,
        AccountDerivationFromMnemonic, AccountDerivationFromPrivateKey, AccountGet,
        AccountGroupCreate, AccountGroupDelete, AccountGroupUpdate, AccountLayoutGet,
        AccountLayoutUpdate, AccountList, AccountMnemonicGenerate, AccountNextIdentity,
        AccountTransactionList, AccountTransactionPending, AccountUpdate, AssetCreate, AssetDelete,
        AssetDiscoverMetadata, AssetGet, AssetList, AssetQuote, AssetUpdate, EndpointCreate,
        EndpointDelete, EndpointGet, EndpointList, EndpointNextIdentity, EndpointStatus,
        EndpointUpdate, NetworkCreate, NetworkDelete, NetworkDiscoverMetadata, NetworkGet,
        NetworkList, NetworkListPresets, NetworkRpcStats, NetworkUpdate, QuoterCreate,
        QuoterDiscover, QuoterGet, QuoterList, QuoterUpdate, SystemPing, TransactionDecode,
        TransactionSimulate, VendorDisable, VendorEnable, VendorListAll, VendorListEnabled,
    },
};
use ts_rs::{Config, TS};

const PREAMBLE: &str = r#"/* Generated from the Rust RpcMethod markers. */

import type {
    Account,
    AccountBalance,
    AccountBalances,
    AccountGroup,
    AccountGroupCreate,
    AccountGroupUpdate,
    AccountLayout,
    AccountLayoutUpdate,
    AccountUpdate,
    Asset,
    AssetMetadataDiscovery,
    AssetUpdate,
    DecodeTransactionRequest,
    DecodeTransactionResponse,
    Network,
    NetworkEndpoint,
    NetworkEndpointUpdate,
    NetworkMetadataDiscovery,
    NetworkUpdate,
    Quoter,
    QuoterCreate,
    QuoterDiscovery,
    QuoterDiscoveryResponse,
    QuoterUpdate,
    RpcErrorObject,
    RpcPoolStats,
    RpcStatus,
    SimulateTransactionRequest,
    SimulateTransactionResponse,
    Tx,
    VendorFlag,
    VendorFlagInfo,
} from "./bindings.gen";

export type RpcIdentity = number | string | null;
export type RpcRequestEnvelope = { jsonrpc: "2.0"; id?: RpcIdentity; method: string; params: Record<string, unknown> };
export type RpcSuccessEnvelope = { jsonrpc: "2.0"; id: RpcIdentity; result: unknown };
export type RpcErrorEnvelope = { jsonrpc: "2.0"; id: RpcIdentity; error: RpcErrorObject };
export type RpcResponseEnvelope = RpcSuccessEnvelope | RpcErrorEnvelope;

"#;

fn push_declaration<T: TS>(output: &mut String) {
    let declaration = T::decl(&Config::default())
        .replace("Record<symbol, never>", "Record<string, never>")
        .replace("fresh: boolean", "fresh?: boolean")
        .replacen("type ", "export type ", 1);
    writeln!(output, "{declaration}").unwrap();
}

fn push_method<M: RpcMethod>(output: &mut String, result: &str)
where
    M::Params: TS,
{
    writeln!(
        output,
        "    \"{}\": {{ params: {}; result: {result} }};",
        M::NAME,
        <M::Params as TS>::name(&Config::default())
    )
    .unwrap();
}

fn main() {
    let mut output = PREAMBLE.to_string();

    push_declaration::<EmptyParams>(&mut output);
    push_declaration::<AccountParams>(&mut output);
    push_declaration::<AssetParams>(&mut output);
    push_declaration::<NetworkParams>(&mut output);
    push_declaration::<AccountAssetParams>(&mut output);
    push_declaration::<AccountAssetBalanceParams>(&mut output);
    push_declaration::<AccountBalancesParams>(&mut output);
    push_declaration::<AccountCreateParams>(&mut output);
    push_declaration::<AccountUpdateParams>(&mut output);
    push_declaration::<LayoutUpdateParams>(&mut output);
    push_declaration::<GroupCreateParams>(&mut output);
    push_declaration::<GroupUpdateParams>(&mut output);
    push_declaration::<GroupParams>(&mut output);
    push_declaration::<DeriveMnemonicInput>(&mut output);
    push_declaration::<DeriveMnemonicParams>(&mut output);
    push_declaration::<DeriveMnemonicResult>(&mut output);
    push_declaration::<DerivePrivateKeyParams>(&mut output);
    push_declaration::<AssetCreateParams>(&mut output);
    push_declaration::<AssetUpdateParams>(&mut output);
    push_declaration::<AssetQuoteParams>(&mut output);
    push_declaration::<NetworkCreateParams>(&mut output);
    push_declaration::<NetworkUpdateParams>(&mut output);
    push_declaration::<EndpointParams>(&mut output);
    push_declaration::<EndpointCreateParams>(&mut output);
    push_declaration::<EndpointUpdateParams>(&mut output);
    push_declaration::<SimulateParams>(&mut output);
    push_declaration::<DecodeParams>(&mut output);
    push_declaration::<QuoterParams>(&mut output);
    push_declaration::<QuoterCreateParams>(&mut output);
    push_declaration::<QuoterUpdateParams>(&mut output);
    push_declaration::<QuoterDiscoverParams>(&mut output);
    push_declaration::<VendorParams>(&mut output);

    output.push_str("\nexport type RpcMethodMap = {\n");
    push_method::<SystemPing>(&mut output, "string");
    push_method::<AccountList>(&mut output, "Account[]");
    push_method::<AccountGet>(&mut output, "Account");
    push_method::<AccountCreate>(&mut output, "Account");
    push_method::<AccountNextIdentity>(&mut output, "number");
    push_method::<AccountUpdate>(&mut output, "Account");
    push_method::<AccountDelete>(&mut output, "null");
    push_method::<AccountAssetList>(&mut output, "string[]");
    push_method::<AccountAssetAdd>(&mut output, "null");
    push_method::<AccountAssetRemove>(&mut output, "null");
    push_method::<AccountAssetBalance>(&mut output, "AccountBalance");
    push_method::<AccountBalanceList>(&mut output, "AccountBalances");
    push_method::<AccountLayoutGet>(&mut output, "AccountLayout");
    push_method::<AccountLayoutUpdate>(&mut output, "AccountLayout");
    push_method::<AccountGroupCreate>(&mut output, "AccountGroup");
    push_method::<AccountGroupUpdate>(&mut output, "AccountGroup");
    push_method::<AccountGroupDelete>(&mut output, "null");
    push_method::<AccountTransactionList>(&mut output, "Tx[]");
    push_method::<AccountTransactionPending>(&mut output, "Tx[]");
    push_method::<AccountMnemonicGenerate>(&mut output, "string");
    push_method::<AccountDerivationDefaultPath>(&mut output, "string");
    push_method::<AccountDerivationFromMnemonic>(&mut output, "DeriveMnemonicResult[]");
    push_method::<AccountDerivationFromPrivateKey>(&mut output, "string");
    push_method::<AssetList>(&mut output, "Asset[]");
    push_method::<AssetGet>(&mut output, "Asset");
    push_method::<AssetCreate>(&mut output, "Asset");
    push_method::<AssetUpdate>(&mut output, "Asset");
    push_method::<AssetDelete>(&mut output, "null");
    push_method::<AssetDiscoverMetadata>(&mut output, "AssetMetadataDiscovery");
    push_method::<AssetQuote>(&mut output, "string");
    push_method::<NetworkList>(&mut output, "Network[]");
    push_method::<NetworkGet>(&mut output, "Network");
    push_method::<NetworkCreate>(&mut output, "Network");
    push_method::<NetworkUpdate>(&mut output, "Network");
    push_method::<NetworkDelete>(&mut output, "null");
    push_method::<NetworkListPresets>(&mut output, "Network[]");
    push_method::<NetworkDiscoverMetadata>(&mut output, "NetworkMetadataDiscovery");
    push_method::<NetworkRpcStats>(&mut output, "RpcPoolStats");
    push_method::<EndpointList>(&mut output, "NetworkEndpoint[]");
    push_method::<EndpointGet>(&mut output, "NetworkEndpoint");
    push_method::<EndpointCreate>(&mut output, "NetworkEndpoint");
    push_method::<EndpointUpdate>(&mut output, "NetworkEndpoint");
    push_method::<EndpointDelete>(&mut output, "null");
    push_method::<EndpointNextIdentity>(&mut output, "number");
    push_method::<EndpointStatus>(&mut output, "RpcStatus");
    push_method::<TransactionSimulate>(&mut output, "SimulateTransactionResponse");
    push_method::<TransactionDecode>(&mut output, "DecodeTransactionResponse");
    push_method::<QuoterList>(&mut output, "Quoter[]");
    push_method::<QuoterGet>(&mut output, "Quoter");
    push_method::<QuoterCreate>(&mut output, "Quoter");
    push_method::<QuoterUpdate>(&mut output, "Quoter");
    push_method::<QuoterDiscover>(&mut output, "QuoterDiscoveryResponse");
    push_method::<VendorListEnabled>(&mut output, "VendorFlag[]");
    push_method::<VendorListAll>(&mut output, "VendorFlagInfo[]");
    push_method::<VendorEnable>(&mut output, "null");
    push_method::<VendorDisable>(&mut output, "null");
    output.push_str(
        "};\n\nexport type RpcMethodName = keyof RpcMethodMap;\nexport type RpcParams<TMethod extends RpcMethodName> = RpcMethodMap[TMethod][\"params\"];\nexport type RpcResult<TMethod extends RpcMethodName> = RpcMethodMap[TMethod][\"result\"];\n",
    );

    let output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../interfaces/web/src/api/rpc-contract.gen.ts");
    fs::write(output_path, output).unwrap();
}
