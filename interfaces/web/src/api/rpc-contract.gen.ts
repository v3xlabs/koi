/* Generated from the Rust RPC contract. Do not edit. */

import type * as RpcBindings from "./bindings.gen";

export type { RpcErrorEnvelope, RpcIdentity, RpcRequestEnvelope, RpcResponseEnvelope, RpcSuccessEnvelope } from "./bindings.gen";

export type RpcMethodMap = {
    "system.ping": { params: RpcBindings.SystemPingRpcParams; result: RpcBindings.SystemPingRpcResult };
    "account.list": { params: RpcBindings.AccountListRpcParams; result: RpcBindings.AccountListRpcResult };
    "account.get": { params: RpcBindings.AccountGetRpcParams; result: RpcBindings.AccountGetRpcResult };
    "account.create": { params: RpcBindings.AccountCreateRpcParams; result: RpcBindings.AccountCreateRpcResult };
    "account.nextIdentity": { params: RpcBindings.AccountNextIdentityRpcParams; result: RpcBindings.AccountNextIdentityRpcResult };
    "account.update": { params: RpcBindings.AccountUpdateRpcParams; result: RpcBindings.AccountUpdateRpcResult };
    "account.delete": { params: RpcBindings.AccountDeleteRpcParams; result: RpcBindings.AccountDeleteRpcResult };
    "account.asset.list": { params: RpcBindings.AccountAssetListRpcParams; result: RpcBindings.AccountAssetListRpcResult };
    "account.asset.add": { params: RpcBindings.AccountAssetAddRpcParams; result: RpcBindings.AccountAssetAddRpcResult };
    "account.asset.remove": { params: RpcBindings.AccountAssetRemoveRpcParams; result: RpcBindings.AccountAssetRemoveRpcResult };
    "account.asset.balance": { params: RpcBindings.AccountAssetBalanceRpcParams; result: RpcBindings.AccountAssetBalanceRpcResult };
    "account.balance.list": { params: RpcBindings.AccountBalanceListRpcParams; result: RpcBindings.AccountBalanceListRpcResult };
    "account.layout.get": { params: RpcBindings.AccountLayoutGetRpcParams; result: RpcBindings.AccountLayoutGetRpcResult };
    "account.layout.update": { params: RpcBindings.AccountLayoutUpdateRpcParams; result: RpcBindings.AccountLayoutUpdateRpcResult };
    "account.group.create": { params: RpcBindings.AccountGroupCreateRpcParams; result: RpcBindings.AccountGroupCreateRpcResult };
    "account.group.update": { params: RpcBindings.AccountGroupUpdateRpcParams; result: RpcBindings.AccountGroupUpdateRpcResult };
    "account.group.delete": { params: RpcBindings.AccountGroupDeleteRpcParams; result: RpcBindings.AccountGroupDeleteRpcResult };
    "account.transaction.list": { params: RpcBindings.AccountTransactionListRpcParams; result: RpcBindings.AccountTransactionListRpcResult };
    "account.transaction.pending": { params: RpcBindings.AccountTransactionPendingRpcParams; result: RpcBindings.AccountTransactionPendingRpcResult };
    "account.mnemonic.generate": { params: RpcBindings.AccountMnemonicGenerateRpcParams; result: RpcBindings.AccountMnemonicGenerateRpcResult };
    "account.derivation.defaultPath": { params: RpcBindings.AccountDerivationDefaultPathRpcParams; result: RpcBindings.AccountDerivationDefaultPathRpcResult };
    "account.derivation.fromMnemonic": { params: RpcBindings.AccountDerivationFromMnemonicRpcParams; result: RpcBindings.AccountDerivationFromMnemonicRpcResult };
    "account.derivation.fromPrivateKey": { params: RpcBindings.AccountDerivationFromPrivateKeyRpcParams; result: RpcBindings.AccountDerivationFromPrivateKeyRpcResult };
    "asset.list": { params: RpcBindings.AssetListRpcParams; result: RpcBindings.AssetListRpcResult };
    "asset.get": { params: RpcBindings.AssetGetRpcParams; result: RpcBindings.AssetGetRpcResult };
    "asset.create": { params: RpcBindings.AssetCreateRpcParams; result: RpcBindings.AssetCreateRpcResult };
    "asset.update": { params: RpcBindings.AssetUpdateRpcParams; result: RpcBindings.AssetUpdateRpcResult };
    "asset.delete": { params: RpcBindings.AssetDeleteRpcParams; result: RpcBindings.AssetDeleteRpcResult };
    "asset.discoverMetadata": { params: RpcBindings.AssetDiscoverMetadataRpcParams; result: RpcBindings.AssetDiscoverMetadataRpcResult };
    "asset.quote": { params: RpcBindings.AssetQuoteRpcParams; result: RpcBindings.AssetQuoteRpcResult };
    "network.list": { params: RpcBindings.NetworkListRpcParams; result: RpcBindings.NetworkListRpcResult };
    "network.get": { params: RpcBindings.NetworkGetRpcParams; result: RpcBindings.NetworkGetRpcResult };
    "network.create": { params: RpcBindings.NetworkCreateRpcParams; result: RpcBindings.NetworkCreateRpcResult };
    "network.update": { params: RpcBindings.NetworkUpdateRpcParams; result: RpcBindings.NetworkUpdateRpcResult };
    "network.delete": { params: RpcBindings.NetworkDeleteRpcParams; result: RpcBindings.NetworkDeleteRpcResult };
    "network.listPresets": { params: RpcBindings.NetworkListPresetsRpcParams; result: RpcBindings.NetworkListPresetsRpcResult };
    "network.discoverMetadata": { params: RpcBindings.NetworkDiscoverMetadataRpcParams; result: RpcBindings.NetworkDiscoverMetadataRpcResult };
    "network.rpcStats": { params: RpcBindings.NetworkRpcStatsRpcParams; result: RpcBindings.NetworkRpcStatsRpcResult };
    "network.endpoint.list": { params: RpcBindings.EndpointListRpcParams; result: RpcBindings.EndpointListRpcResult };
    "network.endpoint.get": { params: RpcBindings.EndpointGetRpcParams; result: RpcBindings.EndpointGetRpcResult };
    "network.endpoint.create": { params: RpcBindings.EndpointCreateRpcParams; result: RpcBindings.EndpointCreateRpcResult };
    "network.endpoint.update": { params: RpcBindings.EndpointUpdateRpcParams; result: RpcBindings.EndpointUpdateRpcResult };
    "network.endpoint.delete": { params: RpcBindings.EndpointDeleteRpcParams; result: RpcBindings.EndpointDeleteRpcResult };
    "network.endpoint.nextIdentity": { params: RpcBindings.EndpointNextIdentityRpcParams; result: RpcBindings.EndpointNextIdentityRpcResult };
    "network.endpoint.status": { params: RpcBindings.EndpointStatusRpcParams; result: RpcBindings.EndpointStatusRpcResult };
    "transaction.simulate": { params: RpcBindings.TransactionSimulateRpcParams; result: RpcBindings.TransactionSimulateRpcResult };
    "transaction.decode": { params: RpcBindings.TransactionDecodeRpcParams; result: RpcBindings.TransactionDecodeRpcResult };
    "quoter.list": { params: RpcBindings.QuoterListRpcParams; result: RpcBindings.QuoterListRpcResult };
    "quoter.get": { params: RpcBindings.QuoterGetRpcParams; result: RpcBindings.QuoterGetRpcResult };
    "quoter.create": { params: RpcBindings.QuoterCreateRpcParams; result: RpcBindings.QuoterCreateRpcResult };
    "quoter.update": { params: RpcBindings.QuoterUpdateRpcParams; result: RpcBindings.QuoterUpdateRpcResult };
    "quoter.discover": { params: RpcBindings.QuoterDiscoverRpcParams; result: RpcBindings.QuoterDiscoverRpcResult };
    "vendor.listEnabled": { params: RpcBindings.VendorListEnabledRpcParams; result: RpcBindings.VendorListEnabledRpcResult };
    "vendor.listAll": { params: RpcBindings.VendorListAllRpcParams; result: RpcBindings.VendorListAllRpcResult };
    "vendor.enable": { params: RpcBindings.VendorEnableRpcParams; result: RpcBindings.VendorEnableRpcResult };
    "vendor.disable": { params: RpcBindings.VendorDisableRpcParams; result: RpcBindings.VendorDisableRpcResult };
};

export type RpcMethodName = keyof RpcMethodMap;
export type RpcParams<TMethod extends RpcMethodName> = RpcMethodMap[TMethod]["params"];
export type RpcResult<TMethod extends RpcMethodName> = RpcMethodMap[TMethod]["result"];
