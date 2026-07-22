/* Generated from the Rust RPC contract. Do not edit. */

import type * as RpcBindings from "./bindings.gen";

export type { RpcErrorEnvelope, RpcIdentity, RpcRequestEnvelope, RpcResponseEnvelope, RpcSuccessEnvelope } from "./bindings.gen";

export type RpcMethodMap = {
    "account.asset.add": { params: RpcBindings.AccountAssetAddRpcParams; result: RpcBindings.AccountAssetAddRpcResult };
    "account.asset.balance": { params: RpcBindings.AccountAssetBalanceRpcParams; result: RpcBindings.AccountAssetBalanceRpcResult };
    "account.asset.list": { params: RpcBindings.AccountAssetListRpcParams; result: RpcBindings.AccountAssetListRpcResult };
    "account.asset.remove": { params: RpcBindings.AccountAssetRemoveRpcParams; result: RpcBindings.AccountAssetRemoveRpcResult };
    "account.balance.list": { params: RpcBindings.AccountBalanceListRpcParams; result: RpcBindings.AccountBalanceListRpcResult };
    "account.create": { params: RpcBindings.AccountCreateRpcParams; result: RpcBindings.AccountCreateRpcResult };
    "account.delete": { params: RpcBindings.AccountDeleteRpcParams; result: RpcBindings.AccountDeleteRpcResult };
    "account.derivation.defaultPath": { params: RpcBindings.AccountDerivationDefaultPathRpcParams; result: RpcBindings.AccountDerivationDefaultPathRpcResult };
    "account.derivation.fromMnemonic": { params: RpcBindings.AccountDerivationFromMnemonicRpcParams; result: RpcBindings.AccountDerivationFromMnemonicRpcResult };
    "account.derivation.fromPrivateKey": { params: RpcBindings.AccountDerivationFromPrivateKeyRpcParams; result: RpcBindings.AccountDerivationFromPrivateKeyRpcResult };
    "account.get": { params: RpcBindings.AccountGetRpcParams; result: RpcBindings.AccountGetRpcResult };
    "account.group.create": { params: RpcBindings.AccountGroupCreateRpcParams; result: RpcBindings.AccountGroupCreateRpcResult };
    "account.group.delete": { params: RpcBindings.AccountGroupDeleteRpcParams; result: RpcBindings.AccountGroupDeleteRpcResult };
    "account.group.update": { params: RpcBindings.AccountGroupUpdateRpcParams; result: RpcBindings.AccountGroupUpdateRpcResult };
    "account.layout.get": { params: RpcBindings.AccountLayoutGetRpcParams; result: RpcBindings.AccountLayoutGetRpcResult };
    "account.layout.update": { params: RpcBindings.AccountLayoutUpdateRpcParams; result: RpcBindings.AccountLayoutUpdateRpcResult };
    "account.list": { params: RpcBindings.AccountListRpcParams; result: RpcBindings.AccountListRpcResult };
    "account.mnemonic.generate": { params: RpcBindings.AccountMnemonicGenerateRpcParams; result: RpcBindings.AccountMnemonicGenerateRpcResult };
    "account.transaction.list": { params: RpcBindings.AccountTransactionListRpcParams; result: RpcBindings.AccountTransactionListRpcResult };
    "account.transaction.pending": { params: RpcBindings.AccountTransactionPendingRpcParams; result: RpcBindings.AccountTransactionPendingRpcResult };
    "account.update": { params: RpcBindings.AccountUpdateRpcParams; result: RpcBindings.AccountUpdateRpcResult };
    "asset.create": { params: RpcBindings.AssetCreateRpcParams; result: RpcBindings.AssetCreateRpcResult };
    "asset.delete": { params: RpcBindings.AssetDeleteRpcParams; result: RpcBindings.AssetDeleteRpcResult };
    "asset.discoverMetadata": { params: RpcBindings.AssetDiscoverMetadataRpcParams; result: RpcBindings.AssetDiscoverMetadataRpcResult };
    "asset.get": { params: RpcBindings.AssetGetRpcParams; result: RpcBindings.AssetGetRpcResult };
    "asset.icon": { params: RpcBindings.AssetIconRpcParams; result: RpcBindings.AssetIconRpcResult };
    "asset.list": { params: RpcBindings.AssetListRpcParams; result: RpcBindings.AssetListRpcResult };
    "asset.quote": { params: RpcBindings.AssetQuoteRpcParams; result: RpcBindings.AssetQuoteRpcResult };
    "asset.update": { params: RpcBindings.AssetUpdateRpcParams; result: RpcBindings.AssetUpdateRpcResult };
    "ens.resolve": { params: RpcBindings.EnsResolveRpcParams; result: RpcBindings.EnsResolveRpcResult };
    "ens.reverse": { params: RpcBindings.EnsReverseRpcParams; result: RpcBindings.EnsReverseRpcResult };
    "network.create": { params: RpcBindings.NetworkCreateRpcParams; result: RpcBindings.NetworkCreateRpcResult };
    "network.delete": { params: RpcBindings.NetworkDeleteRpcParams; result: RpcBindings.NetworkDeleteRpcResult };
    "network.discover": { params: RpcBindings.NetworkDiscoverRpcParams; result: RpcBindings.NetworkDiscoverRpcResult };
    "network.endpoint.create": { params: RpcBindings.EndpointCreateRpcParams; result: RpcBindings.EndpointCreateRpcResult };
    "network.endpoint.delete": { params: RpcBindings.EndpointDeleteRpcParams; result: RpcBindings.EndpointDeleteRpcResult };
    "network.endpoint.get": { params: RpcBindings.EndpointGetRpcParams; result: RpcBindings.EndpointGetRpcResult };
    "network.endpoint.list": { params: RpcBindings.EndpointListRpcParams; result: RpcBindings.EndpointListRpcResult };
    "network.endpoint.status": { params: RpcBindings.EndpointStatusRpcParams; result: RpcBindings.EndpointStatusRpcResult };
    "network.endpoint.update": { params: RpcBindings.EndpointUpdateRpcParams; result: RpcBindings.EndpointUpdateRpcResult };
    "network.get": { params: RpcBindings.NetworkGetRpcParams; result: RpcBindings.NetworkGetRpcResult };
    "network.list": { params: RpcBindings.NetworkListRpcParams; result: RpcBindings.NetworkListRpcResult };
    "network.presets": { params: RpcBindings.NetworkPresetsRpcParams; result: RpcBindings.NetworkPresetsRpcResult };
    "network.stats": { params: RpcBindings.NetworkStatsRpcParams; result: RpcBindings.NetworkStatsRpcResult };
    "network.update": { params: RpcBindings.NetworkUpdateRpcParams; result: RpcBindings.NetworkUpdateRpcResult };
    "quoter.create": { params: RpcBindings.QuoterCreateRpcParams; result: RpcBindings.QuoterCreateRpcResult };
    "quoter.discover": { params: RpcBindings.QuoterDiscoverRpcParams; result: RpcBindings.QuoterDiscoverRpcResult };
    "quoter.get": { params: RpcBindings.QuoterGetRpcParams; result: RpcBindings.QuoterGetRpcResult };
    "quoter.list": { params: RpcBindings.QuoterListRpcParams; result: RpcBindings.QuoterListRpcResult };
    "quoter.update": { params: RpcBindings.QuoterUpdateRpcParams; result: RpcBindings.QuoterUpdateRpcResult };
    "system.ping": { params: RpcBindings.SystemPingRpcParams; result: RpcBindings.SystemPingRpcResult };
    "transaction.decode": { params: RpcBindings.TransactionDecodeRpcParams; result: RpcBindings.TransactionDecodeRpcResult };
    "transaction.simulate": { params: RpcBindings.TransactionSimulateRpcParams; result: RpcBindings.TransactionSimulateRpcResult };
    "vendor.disable": { params: RpcBindings.VendorDisableRpcParams; result: RpcBindings.VendorDisableRpcResult };
    "vendor.enable": { params: RpcBindings.VendorEnableRpcParams; result: RpcBindings.VendorEnableRpcResult };
    "vendor.listAll": { params: RpcBindings.VendorListAllRpcParams; result: RpcBindings.VendorListAllRpcResult };
    "vendor.listEnabled": { params: RpcBindings.VendorListEnabledRpcParams; result: RpcBindings.VendorListEnabledRpcResult };
};

export type RpcMethodName = keyof RpcMethodMap;
export type RpcParams<TMethod extends RpcMethodName> = RpcMethodMap[TMethod]["params"];
export type RpcResult<TMethod extends RpcMethodName> = RpcMethodMap[TMethod]["result"];
