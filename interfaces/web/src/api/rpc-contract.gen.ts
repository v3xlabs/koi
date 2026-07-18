/* Generated from the Rust RpcMethod markers. */

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

export type EmptyParams = Record<string, never>;
export type AccountParams = { account_identity: number, };
export type AssetParams = { asset_identity: string, };
export type NetworkParams = { network_identity: number, };
export type AccountAssetParams = { account_identity: number, asset_identity: string, };
export type AccountAssetBalanceParams = { account_identity: number, asset_identity: string, display_currency: string, };
export type AccountBalancesParams = { account_identity: number, display_currency: string, fresh?: boolean, };
export type AccountCreateParams = { input: Account, };
export type AccountUpdateParams = { account_identity: number, input: AccountUpdate, };
export type LayoutUpdateParams = { input: AccountLayoutUpdate, };
export type GroupCreateParams = { input: AccountGroupCreate, };
export type GroupUpdateParams = { group_identity: number, input: AccountGroupUpdate, };
export type GroupParams = { group_identity: number, };
export type DeriveMnemonicInput = { mnemonic: string, paths: Array<string>, };
export type DeriveMnemonicParams = { input: DeriveMnemonicInput, };
export type DeriveMnemonicResult = { path: string, address: string, };
export type DerivePrivateKeyParams = { input: string, };
export type AssetCreateParams = { input: Asset, };
export type AssetUpdateParams = { asset_identity: string, input: AssetUpdate, };
export type AssetQuoteParams = { asset_identity: string, display_asset?: string | null, };
export type NetworkCreateParams = { input: Network, };
export type NetworkUpdateParams = { network_identity: number, input: NetworkUpdate, };
export type EndpointParams = { network_identity: number, endpoint_identity: number, };
export type EndpointCreateParams = { network_identity: number, input: NetworkEndpoint, };
export type EndpointUpdateParams = { network_identity: number, endpoint_identity: number, input: NetworkEndpointUpdate, };
export type SimulateParams = { network_identity: number, input: SimulateTransactionRequest, };
export type DecodeParams = { network_identity: number, input: DecodeTransactionRequest, };
export type QuoterParams = { quoter_identity: string, };
export type QuoterCreateParams = { input: QuoterCreate, };
export type QuoterUpdateParams = { quoter_identity: string, input: QuoterUpdate, };
export type QuoterDiscoverParams = { input: QuoterDiscovery, };
export type VendorParams = { flag: VendorFlag, };

export type RpcMethodMap = {
    "system.ping": { params: EmptyParams; result: string };
    "account.list": { params: EmptyParams; result: Account[] };
    "account.get": { params: AccountParams; result: Account };
    "account.create": { params: AccountCreateParams; result: Account };
    "account.nextIdentity": { params: EmptyParams; result: number };
    "account.update": { params: AccountUpdateParams; result: Account };
    "account.delete": { params: AccountParams; result: null };
    "account.asset.list": { params: AccountParams; result: string[] };
    "account.asset.add": { params: AccountAssetParams; result: null };
    "account.asset.remove": { params: AccountAssetParams; result: null };
    "account.asset.balance": { params: AccountAssetBalanceParams; result: AccountBalance };
    "account.balance.list": { params: AccountBalancesParams; result: AccountBalances };
    "account.layout.get": { params: EmptyParams; result: AccountLayout };
    "account.layout.update": { params: LayoutUpdateParams; result: AccountLayout };
    "account.group.create": { params: GroupCreateParams; result: AccountGroup };
    "account.group.update": { params: GroupUpdateParams; result: AccountGroup };
    "account.group.delete": { params: GroupParams; result: null };
    "account.transaction.list": { params: AccountParams; result: Tx[] };
    "account.transaction.pending": { params: AccountParams; result: Tx[] };
    "account.mnemonic.generate": { params: EmptyParams; result: string };
    "account.derivation.defaultPath": { params: EmptyParams; result: string };
    "account.derivation.fromMnemonic": { params: DeriveMnemonicParams; result: DeriveMnemonicResult[] };
    "account.derivation.fromPrivateKey": { params: DerivePrivateKeyParams; result: string };
    "asset.list": { params: EmptyParams; result: Asset[] };
    "asset.get": { params: AssetParams; result: Asset };
    "asset.create": { params: AssetCreateParams; result: Asset };
    "asset.update": { params: AssetUpdateParams; result: Asset };
    "asset.delete": { params: AssetParams; result: null };
    "asset.discoverMetadata": { params: AssetParams; result: AssetMetadataDiscovery };
    "asset.quote": { params: AssetQuoteParams; result: string };
    "network.list": { params: EmptyParams; result: Network[] };
    "network.get": { params: NetworkParams; result: Network };
    "network.create": { params: NetworkCreateParams; result: Network };
    "network.update": { params: NetworkUpdateParams; result: Network };
    "network.delete": { params: NetworkParams; result: null };
    "network.listPresets": { params: EmptyParams; result: Network[] };
    "network.discoverMetadata": { params: NetworkParams; result: NetworkMetadataDiscovery };
    "network.rpcStats": { params: NetworkParams; result: RpcPoolStats };
    "network.endpoint.list": { params: NetworkParams; result: NetworkEndpoint[] };
    "network.endpoint.get": { params: EndpointParams; result: NetworkEndpoint };
    "network.endpoint.create": { params: EndpointCreateParams; result: NetworkEndpoint };
    "network.endpoint.update": { params: EndpointUpdateParams; result: NetworkEndpoint };
    "network.endpoint.delete": { params: EndpointParams; result: null };
    "network.endpoint.nextIdentity": { params: NetworkParams; result: number };
    "network.endpoint.status": { params: EndpointParams; result: RpcStatus };
    "transaction.simulate": { params: SimulateParams; result: SimulateTransactionResponse };
    "transaction.decode": { params: DecodeParams; result: DecodeTransactionResponse };
    "quoter.list": { params: EmptyParams; result: Quoter[] };
    "quoter.get": { params: QuoterParams; result: Quoter };
    "quoter.create": { params: QuoterCreateParams; result: Quoter };
    "quoter.update": { params: QuoterUpdateParams; result: Quoter };
    "quoter.discover": { params: QuoterDiscoverParams; result: QuoterDiscoveryResponse };
    "vendor.listEnabled": { params: EmptyParams; result: VendorFlag[] };
    "vendor.listAll": { params: EmptyParams; result: VendorFlagInfo[] };
    "vendor.enable": { params: VendorParams; result: null };
    "vendor.disable": { params: VendorParams; result: null };
};

export type RpcMethodName = keyof RpcMethodMap;
export type RpcParams<TMethod extends RpcMethodName> = RpcMethodMap[TMethod]["params"];
export type RpcResult<TMethod extends RpcMethodName> = RpcMethodMap[TMethod]["result"];
