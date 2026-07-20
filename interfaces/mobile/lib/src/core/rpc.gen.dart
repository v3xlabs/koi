// Generated from the Rust RPC contract. Do not edit.
// ignore_for_file: curly_braces_in_flow_control_structures

import 'dart:convert';

import 'bridge/api.dart' as bridge;
import 'rpc_models.gen.dart';

final class RpcException implements Exception {
  const RpcException(this.code, this.message);
  final int code;
  final String message;
  @override
  String toString() => 'RPC error $code: $message';
}

final class RpcClient {
  RpcClient(this._client);
  final bridge.InProcessClient _client;
  var _nextId = 0;

  Future<T> _call<T>(
    String method,
    Map<String, Object?> params,
    T Function(Object?) decode,
  ) async {
    final id = ++_nextId;
    final response = await bridge.processMessage(
      client: _client,
      message: jsonEncode(<String, Object?>{
        'jsonrpc': '2.0',
        'id': id,
        'method': method,
        'params': params,
      }),
    );
    if (response == null) throw StateError('$method returned no response');
    final envelope = jsonDecode(response) as Map<String, Object?>;
    final error = envelope['error'];
    if (error is Map<String, Object?>)
      throw RpcException(error['code'] as int, error['message'] as String);
    return decode(envelope['result']);
  }

  Future<void> accountAssetAdd(AccountAssetParams params) =>
      _call('account.asset.add', params.toJson(), (_) {});
  Future<AccountBalance> accountAssetBalance(
    AccountAssetBalanceParams params,
  ) => _call(
    'account.asset.balance',
    params.toJson(),
    (value) => decodeAccountBalance(value),
  );
  Future<List<AssetIdentity>> accountAssetList(AccountParams params) => _call(
    'account.asset.list',
    params.toJson(),
    (value) => decodeRpcList(value, (value) => decodeAssetIdentity(value)),
  );
  Future<void> accountAssetRemove(AccountAssetParams params) =>
      _call('account.asset.remove', params.toJson(), (_) {});
  Future<AccountBalances> accountBalanceList(AccountBalancesParams params) =>
      _call(
        'account.balance.list',
        params.toJson(),
        (value) => decodeAccountBalances(value),
      );
  Future<Account> accountCreate(AccountCreateParams params) =>
      _call('account.create', params.toJson(), (value) => decodeAccount(value));
  Future<void> accountDelete(AccountParams params) =>
      _call('account.delete', params.toJson(), (_) {});
  Future<String> accountDerivationDefaultPath() => _call(
    'account.derivation.defaultPath',
    const <String, Object?>{},
    (value) => value as String,
  );
  Future<List<DeriveMnemonicResult>> accountDerivationFromMnemonic(
    DeriveMnemonicParams params,
  ) => _call(
    'account.derivation.fromMnemonic',
    params.toJson(),
    (value) =>
        decodeRpcList(value, (value) => decodeDeriveMnemonicResult(value)),
  );
  Future<String> accountDerivationFromPrivateKey(
    DerivePrivateKeyParams params,
  ) => _call(
    'account.derivation.fromPrivateKey',
    params.toJson(),
    (value) => value as String,
  );
  Future<Account> accountGet(AccountParams params) =>
      _call('account.get', params.toJson(), (value) => decodeAccount(value));
  Future<AccountGroup> accountGroupCreate(GroupCreateParams params) => _call(
    'account.group.create',
    params.toJson(),
    (value) => decodeAccountGroup(value),
  );
  Future<void> accountGroupDelete(GroupParams params) =>
      _call('account.group.delete', params.toJson(), (_) {});
  Future<AccountGroup> accountGroupUpdate(GroupUpdateParams params) => _call(
    'account.group.update',
    params.toJson(),
    (value) => decodeAccountGroup(value),
  );
  Future<AccountLayout> accountLayoutGet() => _call(
    'account.layout.get',
    const <String, Object?>{},
    (value) => decodeAccountLayout(value),
  );
  Future<AccountLayout> accountLayoutUpdate(LayoutUpdateParams params) => _call(
    'account.layout.update',
    params.toJson(),
    (value) => decodeAccountLayout(value),
  );
  Future<List<Account>> accountList() => _call(
    'account.list',
    const <String, Object?>{},
    (value) => decodeRpcList(value, (value) => decodeAccount(value)),
  );
  Future<String> accountMnemonicGenerate() => _call(
    'account.mnemonic.generate',
    const <String, Object?>{},
    (value) => value as String,
  );
  Future<AccountIdentity> accountNextIdentity() => _call(
    'account.nextIdentity',
    const <String, Object?>{},
    (value) => decodeAccountIdentity(value),
  );
  Future<List<Tx>> accountTransactionList(AccountParams params) => _call(
    'account.transaction.list',
    params.toJson(),
    (value) => decodeRpcList(value, (value) => decodeTx(value)),
  );
  Future<List<Tx>> accountTransactionPending(AccountParams params) => _call(
    'account.transaction.pending',
    params.toJson(),
    (value) => decodeRpcList(value, (value) => decodeTx(value)),
  );
  Future<Account> accountUpdate(AccountUpdateParams params) =>
      _call('account.update', params.toJson(), (value) => decodeAccount(value));
  Future<Asset> assetCreate(AssetCreateParams params) =>
      _call('asset.create', params.toJson(), (value) => decodeAsset(value));
  Future<void> assetDelete(AssetParams params) =>
      _call('asset.delete', params.toJson(), (_) {});
  Future<AssetMetadataDiscovery> assetDiscoverMetadata(AssetParams params) =>
      _call(
        'asset.discoverMetadata',
        params.toJson(),
        (value) => decodeAssetMetadataDiscovery(value),
      );
  Future<Asset> assetGet(AssetParams params) =>
      _call('asset.get', params.toJson(), (value) => decodeAsset(value));
  Future<List<Asset>> assetList() => _call(
    'asset.list',
    const <String, Object?>{},
    (value) => decodeRpcList(value, (value) => decodeAsset(value)),
  );
  Future<String> assetQuote(AssetQuoteParams params) =>
      _call('asset.quote', params.toJson(), (value) => value as String);
  Future<Asset> assetUpdate(AssetUpdateParams params) =>
      _call('asset.update', params.toJson(), (value) => decodeAsset(value));
  Future<Network> networkCreate(NetworkCreateParams params) =>
      _call('network.create', params.toJson(), (value) => decodeNetwork(value));
  Future<void> networkDelete(NetworkParams params) =>
      _call('network.delete', params.toJson(), (_) {});
  Future<NetworkMetadataDiscovery> networkDiscoverMetadata(
    NetworkParams params,
  ) => _call(
    'network.discoverMetadata',
    params.toJson(),
    (value) => decodeNetworkMetadataDiscovery(value),
  );
  Future<NetworkEndpoint> endpointCreate(EndpointCreateParams params) => _call(
    'network.endpoint.create',
    params.toJson(),
    (value) => decodeNetworkEndpoint(value),
  );
  Future<void> endpointDelete(EndpointParams params) =>
      _call('network.endpoint.delete', params.toJson(), (_) {});
  Future<NetworkEndpoint> endpointGet(EndpointParams params) => _call(
    'network.endpoint.get',
    params.toJson(),
    (value) => decodeNetworkEndpoint(value),
  );
  Future<List<NetworkEndpoint>> endpointList(NetworkParams params) => _call(
    'network.endpoint.list',
    params.toJson(),
    (value) => decodeRpcList(value, (value) => decodeNetworkEndpoint(value)),
  );
  Future<num> endpointNextIdentity(NetworkParams params) => _call(
    'network.endpoint.nextIdentity',
    params.toJson(),
    (value) => value as num,
  );
  Future<RpcStatus> endpointStatus(EndpointParams params) => _call(
    'network.endpoint.status',
    params.toJson(),
    (value) => decodeRpcStatus(value),
  );
  Future<NetworkEndpoint> endpointUpdate(EndpointUpdateParams params) => _call(
    'network.endpoint.update',
    params.toJson(),
    (value) => decodeNetworkEndpoint(value),
  );
  Future<Network> networkGet(NetworkParams params) =>
      _call('network.get', params.toJson(), (value) => decodeNetwork(value));
  Future<List<Network>> networkList() => _call(
    'network.list',
    const <String, Object?>{},
    (value) => decodeRpcList(value, (value) => decodeNetwork(value)),
  );
  Future<List<Network>> networkListPresets() => _call(
    'network.listPresets',
    const <String, Object?>{},
    (value) => decodeRpcList(value, (value) => decodeNetwork(value)),
  );
  Future<RpcPoolStats> networkRpcStats(NetworkParams params) => _call(
    'network.rpcStats',
    params.toJson(),
    (value) => decodeRpcPoolStats(value),
  );
  Future<Network> networkUpdate(NetworkUpdateParams params) =>
      _call('network.update', params.toJson(), (value) => decodeNetwork(value));
  Future<Quoter> quoterCreate(QuoterCreateParams params) =>
      _call('quoter.create', params.toJson(), (value) => decodeQuoter(value));
  Future<QuoterDiscoveryResponse> quoterDiscover(QuoterDiscoverParams params) =>
      _call(
        'quoter.discover',
        params.toJson(),
        (value) => decodeQuoterDiscoveryResponse(value),
      );
  Future<Quoter> quoterGet(QuoterParams params) =>
      _call('quoter.get', params.toJson(), (value) => decodeQuoter(value));
  Future<List<Quoter>> quoterList() => _call(
    'quoter.list',
    const <String, Object?>{},
    (value) => decodeRpcList(value, (value) => decodeQuoter(value)),
  );
  Future<Quoter> quoterUpdate(QuoterUpdateParams params) =>
      _call('quoter.update', params.toJson(), (value) => decodeQuoter(value));
  Future<String> systemPing() => _call(
    'system.ping',
    const <String, Object?>{},
    (value) => value as String,
  );
  Future<DecodeTransactionResponse> transactionDecode(DecodeParams params) =>
      _call(
        'transaction.decode',
        params.toJson(),
        (value) => decodeDecodeTransactionResponse(value),
      );
  Future<SimulateTransactionResponse> transactionSimulate(
    SimulateParams params,
  ) => _call(
    'transaction.simulate',
    params.toJson(),
    (value) => decodeSimulateTransactionResponse(value),
  );
  Future<void> vendorDisable(VendorParams params) =>
      _call('vendor.disable', params.toJson(), (_) {});
  Future<void> vendorEnable(VendorParams params) =>
      _call('vendor.enable', params.toJson(), (_) {});
  Future<List<VendorFlagInfo>> vendorListAll() => _call(
    'vendor.listAll',
    const <String, Object?>{},
    (value) => decodeRpcList(value, (value) => decodeVendorFlagInfo(value)),
  );
  Future<List<VendorFlag>> vendorListEnabled() => _call(
    'vendor.listEnabled',
    const <String, Object?>{},
    (value) => decodeRpcList(value, (value) => decodeVendorFlag(value)),
  );
}
