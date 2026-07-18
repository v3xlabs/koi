/* Generated method wrappers: Rust method markers are canonical. */
import { z } from "zod";

import {
    accountBalanceSchema, accountBalancesSchema, accountGroupSchema, accountLayoutSchema,
    accountSchema, assetMetadataDiscoverySchema, assetSchema, decodeTransactionResponseSchema,
    deriveMnemonicResultSchema,
    networkEndpointSchema, networkMetadataDiscoverySchema, networkSchema, quoterDiscoveryResponseSchema,
    quoterSchema, rpcPoolStatsSchema, rpcStatusSchema, simulateTransactionResponseSchema, txSchema,
    vendorFlagInfoSchema, vendorFlagSchema,
} from "./bindings.zod.gen";
import type {
    Account, AccountGroupCreate, AccountGroupUpdate, AccountLayoutUpdate, AccountUpdate, Asset,
    AssetUpdate, Network, NetworkEndpoint, NetworkEndpointUpdate, NetworkUpdate, QuoterCreate,
    QuoterDiscovery, QuoterUpdate, SimulateTransactionRequest, DecodeTransactionRequest, VendorFlag,
} from "./bindings.gen";
import { createRpcClient } from "./rpc-transport";
import type { RpcMethodName, RpcParams, RpcResult } from "./rpc-contract.gen";

export const rpcTransport = createRpcClient();
const empty = {};
const unitSchema = z.null();

export type RpcBatchItem<TMethod extends RpcMethodName> = {
    method: TMethod;
    params: RpcParams<TMethod>;
    parse: (value: unknown) => RpcResult<TMethod>;
};

export const rpcBatch = async <TMethod extends RpcMethodName>(calls: readonly RpcBatchItem<TMethod>[]): Promise<RpcResult<TMethod>[]> => (
    await rpcTransport.batch(calls)
);

export const rpc = {
    systemPing: () => rpcTransport.call("system.ping", empty, value => z.string().parse(value)),
    accountList: () => rpcTransport.call("account.list", empty, value => z.array(accountSchema).parse(value)),
    accountGet: (account_identity: number) => rpcTransport.call("account.get", { account_identity }, value => accountSchema.parse(value)),
    accountCreate: (input: Account) => rpcTransport.call("account.create", { input }, value => accountSchema.parse(value)),
    accountNextIdentity: () => rpcTransport.call("account.nextIdentity", empty, value => z.number().parse(value)),
    accountUpdate: (account_identity: number, input: AccountUpdate) => rpcTransport.call("account.update", { account_identity, input }, value => accountSchema.parse(value)),
    accountDelete: (account_identity: number) => rpcTransport.call("account.delete", { account_identity }, value => unitSchema.parse(value)),
    accountAssetList: (account_identity: number) => rpcTransport.call("account.asset.list", { account_identity }, value => z.array(z.string()).parse(value)),
    accountAssetAdd: (account_identity: number, asset_identity: string) => rpcTransport.call("account.asset.add", { account_identity, asset_identity }, value => unitSchema.parse(value)),
    accountAssetRemove: (account_identity: number, asset_identity: string) => rpcTransport.call("account.asset.remove", { account_identity, asset_identity }, value => unitSchema.parse(value)),
    accountAssetBalance: (account_identity: number, asset_identity: string, display_currency: string) => rpcTransport.call("account.asset.balance", { account_identity, asset_identity, display_currency }, value => accountBalanceSchema.parse(value)),
    accountBalanceList: (account_identity: number, display_currency: string, fresh = false) => rpcTransport.call("account.balance.list", { account_identity, display_currency, fresh }, value => accountBalancesSchema.parse(value)),
    accountLayoutGet: () => rpcTransport.call("account.layout.get", empty, value => accountLayoutSchema.parse(value)),
    accountLayoutUpdate: (input: AccountLayoutUpdate) => rpcTransport.call("account.layout.update", { input }, value => accountLayoutSchema.parse(value)),
    accountGroupCreate: (input: AccountGroupCreate) => rpcTransport.call("account.group.create", { input }, value => accountGroupSchema.parse(value)),
    accountGroupUpdate: (group_identity: number, input: AccountGroupUpdate) => rpcTransport.call("account.group.update", { group_identity, input }, value => accountGroupSchema.parse(value)),
    accountGroupDelete: (group_identity: number) => rpcTransport.call("account.group.delete", { group_identity }, value => unitSchema.parse(value)),
    accountTransactionList: (account_identity: number) => rpcTransport.call("account.transaction.list", { account_identity }, value => z.array(txSchema).parse(value)),
    accountTransactionPending: (account_identity: number) => rpcTransport.call("account.transaction.pending", { account_identity }, value => z.array(txSchema).parse(value)),
    accountMnemonicGenerate: () => rpcTransport.call("account.mnemonic.generate", empty, value => z.string().parse(value)),
    accountDerivationDefaultPath: () => rpcTransport.call("account.derivation.defaultPath", empty, value => z.string().parse(value)),
    accountDerivationFromMnemonic: (input: { mnemonic: string; paths: string[] }) => rpcTransport.call("account.derivation.fromMnemonic", { input }, value => z.array(deriveMnemonicResultSchema).parse(value)),
    accountDerivationFromPrivateKey: (input: string) => rpcTransport.call("account.derivation.fromPrivateKey", { input }, value => z.string().parse(value)),
    assetList: () => rpcTransport.call("asset.list", empty, value => z.array(assetSchema).parse(value)),
    assetGet: (asset_identity: string) => rpcTransport.call("asset.get", { asset_identity }, value => assetSchema.parse(value)),
    assetCreate: (input: Asset) => rpcTransport.call("asset.create", { input }, value => assetSchema.parse(value)),
    assetUpdate: (asset_identity: string, input: AssetUpdate) => rpcTransport.call("asset.update", { asset_identity, input }, value => assetSchema.parse(value)),
    assetDelete: (asset_identity: string) => rpcTransport.call("asset.delete", { asset_identity }, value => unitSchema.parse(value)),
    assetDiscoverMetadata: (asset_identity: string) => rpcTransport.call("asset.discoverMetadata", { asset_identity }, value => assetMetadataDiscoverySchema.parse(value)),
    assetQuote: (asset_identity: string, display_asset?: string) => rpcTransport.call("asset.quote", { asset_identity, display_asset }, value => z.string().parse(value)),
    networkList: () => rpcTransport.call("network.list", empty, value => z.array(networkSchema).parse(value)),
    networkGet: (network_identity: number) => rpcTransport.call("network.get", { network_identity }, value => networkSchema.parse(value)),
    networkCreate: (input: Network) => rpcTransport.call("network.create", { input }, value => networkSchema.parse(value)),
    networkUpdate: (network_identity: number, input: NetworkUpdate) => rpcTransport.call("network.update", { network_identity, input }, value => networkSchema.parse(value)),
    networkDelete: (network_identity: number) => rpcTransport.call("network.delete", { network_identity }, value => unitSchema.parse(value)),
    networkListPresets: () => rpcTransport.call("network.listPresets", empty, value => z.array(networkSchema).parse(value)),
    networkDiscoverMetadata: (network_identity: number) => rpcTransport.call("network.discoverMetadata", { network_identity }, value => networkMetadataDiscoverySchema.parse(value)),
    networkRpcStats: (network_identity: number) => rpcTransport.call("network.rpcStats", { network_identity }, value => rpcPoolStatsSchema.parse(value)),
    endpointList: (network_identity: number) => rpcTransport.call("network.endpoint.list", { network_identity }, value => z.array(networkEndpointSchema).parse(value)),
    endpointGet: (network_identity: number, endpoint_identity: number) => rpcTransport.call("network.endpoint.get", { network_identity, endpoint_identity }, value => networkEndpointSchema.parse(value)),
    endpointCreate: (network_identity: number, input: NetworkEndpoint) => rpcTransport.call("network.endpoint.create", { network_identity, input }, value => networkEndpointSchema.parse(value)),
    endpointUpdate: (network_identity: number, endpoint_identity: number, input: NetworkEndpointUpdate) => rpcTransport.call("network.endpoint.update", { network_identity, endpoint_identity, input }, value => networkEndpointSchema.parse(value)),
    endpointDelete: (network_identity: number, endpoint_identity: number) => rpcTransport.call("network.endpoint.delete", { network_identity, endpoint_identity }, value => unitSchema.parse(value)),
    endpointNextIdentity: (network_identity: number) => rpcTransport.call("network.endpoint.nextIdentity", { network_identity }, value => z.number().parse(value)),
    endpointStatus: (network_identity: number, endpoint_identity: number) => rpcTransport.call("network.endpoint.status", { network_identity, endpoint_identity }, value => rpcStatusSchema.parse(value)),
    transactionSimulate: (network_identity: number, input: SimulateTransactionRequest) => rpcTransport.call("transaction.simulate", { network_identity, input }, value => simulateTransactionResponseSchema.parse(value)),
    transactionDecode: (network_identity: number, input: DecodeTransactionRequest) => rpcTransport.call("transaction.decode", { network_identity, input }, value => decodeTransactionResponseSchema.parse(value)),
    quoterList: () => rpcTransport.call("quoter.list", empty, value => z.array(quoterSchema).parse(value)),
    quoterGet: (quoter_identity: string) => rpcTransport.call("quoter.get", { quoter_identity }, value => quoterSchema.parse(value)),
    quoterCreate: (input: QuoterCreate) => rpcTransport.call("quoter.create", { input }, value => quoterSchema.parse(value)),
    quoterUpdate: (quoter_identity: string, input: QuoterUpdate) => rpcTransport.call("quoter.update", { quoter_identity, input }, value => quoterSchema.parse(value)),
    quoterDiscover: (input: QuoterDiscovery) => rpcTransport.call("quoter.discover", { input }, value => quoterDiscoveryResponseSchema.parse(value)),
    vendorListEnabled: () => rpcTransport.call("vendor.listEnabled", empty, value => z.array(vendorFlagSchema).parse(value)),
    vendorListAll: () => rpcTransport.call("vendor.listAll", empty, value => z.array(vendorFlagInfoSchema).parse(value)),
    vendorEnable: (flag: VendorFlag) => rpcTransport.call("vendor.enable", { flag }, value => unitSchema.parse(value)),
    vendorDisable: (flag: VendorFlag) => rpcTransport.call("vendor.disable", { flag }, value => unitSchema.parse(value)),
};
