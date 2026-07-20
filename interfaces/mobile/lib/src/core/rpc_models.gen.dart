// Generated from the Rust RPC contract. Do not edit.
// ignore_for_file: use_null_aware_elements

List<T> decodeRpcList<T>(Object? value, T Function(Object?) decode) =>
    (value as List<Object?>).map(decode).toList();

final class Account {
  const Account({
    required this.accountIdentity,
    required this.name,
    required this.networks,
    required this.metadata,
    this.groupId,
    required this.displayOrder,
  });
  final AccountIdentity accountIdentity;
  final String name;
  final List<NetworkIdentity> networks;
  final WalletType metadata;
  final GroupIdentity? groupId;
  final num displayOrder;
  factory Account.fromJson(Map<String, Object?> json) => Account(
    accountIdentity: json['account_identity'] as num,
    name: json['name'] as String,
    networks: decodeRpcList(json['networks'], (value) => value as num),
    metadata: json['metadata'] as Map<String, Object?>,
    groupId: json.containsKey('group_id') ? json['group_id'] as num : null,
    displayOrder: json['display_order'] as num,
  );
  Map<String, Object?> toJson() => {
    'account_identity': accountIdentity,
    'name': name,
    'networks': networks.map((value) => value).toList(),
    'metadata': metadata,
    if (groupId != null) 'group_id': groupId!,
    'display_order': displayOrder,
  };
}

final class AccountAssetBalanceParams {
  const AccountAssetBalanceParams({
    required this.accountIdentity,
    required this.assetIdentity,
    required this.displayCurrency,
  });
  final AccountIdentity accountIdentity;
  final AssetIdentity assetIdentity;
  final AssetIdentity displayCurrency;
  factory AccountAssetBalanceParams.fromJson(Map<String, Object?> json) =>
      AccountAssetBalanceParams(
        accountIdentity: json['account_identity'] as num,
        assetIdentity: json['asset_identity'] as String,
        displayCurrency: json['display_currency'] as String,
      );
  Map<String, Object?> toJson() => {
    'account_identity': accountIdentity,
    'asset_identity': assetIdentity,
    'display_currency': displayCurrency,
  };
}

final class AccountAssetParams {
  const AccountAssetParams({
    required this.accountIdentity,
    required this.assetIdentity,
  });
  final AccountIdentity accountIdentity;
  final AssetIdentity assetIdentity;
  factory AccountAssetParams.fromJson(Map<String, Object?> json) =>
      AccountAssetParams(
        accountIdentity: json['account_identity'] as num,
        assetIdentity: json['asset_identity'] as String,
      );
  Map<String, Object?> toJson() => {
    'account_identity': accountIdentity,
    'asset_identity': assetIdentity,
  };
}

final class AccountBalance {
  const AccountBalance({
    required this.assetIdentity,
    this.balance,
    this.balanceError,
    this.assetQuote,
    this.assetQuoteError,
    this.asset24hQuote,
    this.asset24hQuoteError,
    this.balanceQuote,
    this.balanceQuoteError,
    required this.updatedAt,
  });
  final AssetIdentity assetIdentity;
  final String? balance;
  final String? balanceError;
  final String? assetQuote;
  final String? assetQuoteError;
  final String? asset24hQuote;
  final String? asset24hQuoteError;
  final String? balanceQuote;
  final String? balanceQuoteError;
  final String updatedAt;
  factory AccountBalance.fromJson(Map<String, Object?> json) => AccountBalance(
    assetIdentity: json['asset_identity'] as String,
    balance: json.containsKey('balance') ? json['balance'] as String : null,
    balanceError: json.containsKey('balance_error')
        ? json['balance_error'] as String
        : null,
    assetQuote: json.containsKey('asset_quote')
        ? json['asset_quote'] as String
        : null,
    assetQuoteError: json.containsKey('asset_quote_error')
        ? json['asset_quote_error'] as String
        : null,
    asset24hQuote: json.containsKey('asset_24h_quote')
        ? json['asset_24h_quote'] as String
        : null,
    asset24hQuoteError: json.containsKey('asset_24h_quote_error')
        ? json['asset_24h_quote_error'] as String
        : null,
    balanceQuote: json.containsKey('balance_quote')
        ? json['balance_quote'] as String
        : null,
    balanceQuoteError: json.containsKey('balance_quote_error')
        ? json['balance_quote_error'] as String
        : null,
    updatedAt: json['updated_at'] as String,
  );
  Map<String, Object?> toJson() => {
    'asset_identity': assetIdentity,
    if (balance != null) 'balance': balance!,
    if (balanceError != null) 'balance_error': balanceError!,
    if (assetQuote != null) 'asset_quote': assetQuote!,
    if (assetQuoteError != null) 'asset_quote_error': assetQuoteError!,
    if (asset24hQuote != null) 'asset_24h_quote': asset24hQuote!,
    if (asset24hQuoteError != null)
      'asset_24h_quote_error': asset24hQuoteError!,
    if (balanceQuote != null) 'balance_quote': balanceQuote!,
    if (balanceQuoteError != null) 'balance_quote_error': balanceQuoteError!,
    'updated_at': updatedAt,
  };
}

final class AccountBalances {
  const AccountBalances({
    required this.balances,
    this.totalQuote,
    required this.updatedAt,
    required this.asset,
    required this.errors,
  });
  final List<AccountBalance> balances;
  final String? totalQuote;
  final String updatedAt;
  final AssetIdentity asset;
  final List<String> errors;
  factory AccountBalances.fromJson(Map<String, Object?> json) =>
      AccountBalances(
        balances: decodeRpcList(
          json['balances'],
          (value) => AccountBalance.fromJson(value as Map<String, Object?>),
        ),
        totalQuote: json.containsKey('total_quote')
            ? json['total_quote'] as String
            : null,
        updatedAt: json['updated_at'] as String,
        asset: json['asset'] as String,
        errors: decodeRpcList(json['errors'], (value) => value as String),
      );
  Map<String, Object?> toJson() => {
    'balances': balances.map((value) => value.toJson()).toList(),
    if (totalQuote != null) 'total_quote': totalQuote!,
    'updated_at': updatedAt,
    'asset': asset,
    'errors': errors.map((value) => value).toList(),
  };
}

final class AccountBalancesParams {
  const AccountBalancesParams({
    required this.accountIdentity,
    required this.displayCurrency,
    this.fresh,
  });
  final AccountIdentity accountIdentity;
  final AssetIdentity displayCurrency;
  final bool? fresh;
  factory AccountBalancesParams.fromJson(Map<String, Object?> json) =>
      AccountBalancesParams(
        accountIdentity: json['account_identity'] as num,
        displayCurrency: json['display_currency'] as String,
        fresh: json.containsKey('fresh') ? json['fresh'] as bool : null,
      );
  Map<String, Object?> toJson() => {
    'account_identity': accountIdentity,
    'display_currency': displayCurrency,
    if (fresh != null) 'fresh': fresh!,
  };
}

final class AccountCreateParams {
  const AccountCreateParams({required this.input});
  final Account input;
  factory AccountCreateParams.fromJson(Map<String, Object?> json) =>
      AccountCreateParams(
        input: Account.fromJson(json['input'] as Map<String, Object?>),
      );
  Map<String, Object?> toJson() => {'input': input.toJson()};
}

final class AccountGroup {
  const AccountGroup({
    required this.groupIdentity,
    required this.name,
    required this.displayOrder,
  });
  final GroupIdentity groupIdentity;
  final String name;
  final num displayOrder;
  factory AccountGroup.fromJson(Map<String, Object?> json) => AccountGroup(
    groupIdentity: json['group_identity'] as num,
    name: json['name'] as String,
    displayOrder: json['display_order'] as num,
  );
  Map<String, Object?> toJson() => {
    'group_identity': groupIdentity,
    'name': name,
    'display_order': displayOrder,
  };
}

final class AccountGroupCreate {
  const AccountGroupCreate({required this.name});
  final String name;
  factory AccountGroupCreate.fromJson(Map<String, Object?> json) =>
      AccountGroupCreate(name: json['name'] as String);
  Map<String, Object?> toJson() => {'name': name};
}

final class AccountGroupUpdate {
  const AccountGroupUpdate({this.name});
  final String? name;
  factory AccountGroupUpdate.fromJson(Map<String, Object?> json) =>
      AccountGroupUpdate(
        name: json.containsKey('name') ? json['name'] as String : null,
      );
  Map<String, Object?> toJson() => {if (name != null) 'name': name!};
}

typedef AccountIdentity = num;

final class AccountLayout {
  const AccountLayout({required this.groups, required this.accounts});
  final List<AccountGroup> groups;
  final List<Account> accounts;
  factory AccountLayout.fromJson(Map<String, Object?> json) => AccountLayout(
    groups: decodeRpcList(
      json['groups'],
      (value) => AccountGroup.fromJson(value as Map<String, Object?>),
    ),
    accounts: decodeRpcList(
      json['accounts'],
      (value) => Account.fromJson(value as Map<String, Object?>),
    ),
  );
  Map<String, Object?> toJson() => {
    'groups': groups.map((value) => value.toJson()).toList(),
    'accounts': accounts.map((value) => value.toJson()).toList(),
  };
}

final class AccountLayoutAccountEntry {
  const AccountLayoutAccountEntry({
    required this.accountIdentity,
    this.groupId,
    required this.displayOrder,
  });
  final AccountIdentity accountIdentity;
  final GroupIdentity? groupId;
  final num displayOrder;
  factory AccountLayoutAccountEntry.fromJson(Map<String, Object?> json) =>
      AccountLayoutAccountEntry(
        accountIdentity: json['account_identity'] as num,
        groupId: json.containsKey('group_id') ? json['group_id'] as num : null,
        displayOrder: json['display_order'] as num,
      );
  Map<String, Object?> toJson() => {
    'account_identity': accountIdentity,
    if (groupId != null) 'group_id': groupId!,
    'display_order': displayOrder,
  };
}

final class AccountLayoutGroupEntry {
  const AccountLayoutGroupEntry({
    required this.groupIdentity,
    required this.name,
    required this.displayOrder,
  });
  final GroupIdentity groupIdentity;
  final String name;
  final num displayOrder;
  factory AccountLayoutGroupEntry.fromJson(Map<String, Object?> json) =>
      AccountLayoutGroupEntry(
        groupIdentity: json['group_identity'] as num,
        name: json['name'] as String,
        displayOrder: json['display_order'] as num,
      );
  Map<String, Object?> toJson() => {
    'group_identity': groupIdentity,
    'name': name,
    'display_order': displayOrder,
  };
}

final class AccountLayoutUpdate {
  const AccountLayoutUpdate({required this.groups, required this.accounts});
  final List<AccountLayoutGroupEntry> groups;
  final List<AccountLayoutAccountEntry> accounts;
  factory AccountLayoutUpdate.fromJson(Map<String, Object?> json) =>
      AccountLayoutUpdate(
        groups: decodeRpcList(
          json['groups'],
          (value) =>
              AccountLayoutGroupEntry.fromJson(value as Map<String, Object?>),
        ),
        accounts: decodeRpcList(
          json['accounts'],
          (value) =>
              AccountLayoutAccountEntry.fromJson(value as Map<String, Object?>),
        ),
      );
  Map<String, Object?> toJson() => {
    'groups': groups.map((value) => value.toJson()).toList(),
    'accounts': accounts.map((value) => value.toJson()).toList(),
  };
}

final class AccountParams {
  const AccountParams({required this.accountIdentity});
  final AccountIdentity accountIdentity;
  factory AccountParams.fromJson(Map<String, Object?> json) =>
      AccountParams(accountIdentity: json['account_identity'] as num);
  Map<String, Object?> toJson() => {'account_identity': accountIdentity};
}

final class AccountUpdate {
  const AccountUpdate({this.name, this.networks, this.metadata});
  final String? name;
  final List<NetworkIdentity>? networks;
  final WalletType? metadata;
  factory AccountUpdate.fromJson(Map<String, Object?> json) => AccountUpdate(
    name: json.containsKey('name') ? json['name'] as String : null,
    networks: json.containsKey('networks')
        ? decodeRpcList(json['networks'], (value) => value as num)
        : null,
    metadata: json.containsKey('metadata')
        ? json['metadata'] as Map<String, Object?>
        : null,
  );
  Map<String, Object?> toJson() => {
    if (name != null) 'name': name!,
    if (networks != null) 'networks': networks!.map((value) => value).toList(),
    if (metadata != null) 'metadata': metadata!,
  };
}

final class AccountUpdateParams {
  const AccountUpdateParams({
    required this.accountIdentity,
    required this.input,
  });
  final AccountIdentity accountIdentity;
  final AccountUpdate input;
  factory AccountUpdateParams.fromJson(Map<String, Object?> json) =>
      AccountUpdateParams(
        accountIdentity: json['account_identity'] as num,
        input: AccountUpdate.fromJson(json['input'] as Map<String, Object?>),
      );
  Map<String, Object?> toJson() => {
    'account_identity': accountIdentity,
    'input': input.toJson(),
  };
}

typedef ApiAddress = String;

typedef ApiBytes = String;

typedef ApiU256 = String;

final class Asset {
  const Asset({
    required this.assetIdentity,
    required this.assetName,
    required this.assetSymbol,
    required this.assetDecimals,
    this.assetIconUrl,
  });
  final AssetIdentity assetIdentity;
  final String assetName;
  final String assetSymbol;
  final num assetDecimals;
  final String? assetIconUrl;
  factory Asset.fromJson(Map<String, Object?> json) => Asset(
    assetIdentity: json['asset_identity'] as String,
    assetName: json['asset_name'] as String,
    assetSymbol: json['asset_symbol'] as String,
    assetDecimals: json['asset_decimals'] as num,
    assetIconUrl: json.containsKey('asset_icon_url')
        ? json['asset_icon_url'] as String
        : null,
  );
  Map<String, Object?> toJson() => {
    'asset_identity': assetIdentity,
    'asset_name': assetName,
    'asset_symbol': assetSymbol,
    'asset_decimals': assetDecimals,
    if (assetIconUrl != null) 'asset_icon_url': assetIconUrl!,
  };
}

final class AssetCreateParams {
  const AssetCreateParams({required this.input});
  final Asset input;
  factory AssetCreateParams.fromJson(Map<String, Object?> json) =>
      AssetCreateParams(
        input: Asset.fromJson(json['input'] as Map<String, Object?>),
      );
  Map<String, Object?> toJson() => {'input': input.toJson()};
}

typedef AssetIdentity = String;

final class AssetMetadataDiscovery {
  const AssetMetadataDiscovery({
    required this.assetIdentity,
    required this.options,
  });
  final AssetIdentity assetIdentity;
  final Map<String, AssetMetadataOption> options;
  factory AssetMetadataDiscovery.fromJson(Map<String, Object?> json) =>
      AssetMetadataDiscovery(
        assetIdentity: json['asset_identity'] as String,
        options: (json['options'] as Map<String, Object?>).map(
          (key, value) => MapEntry(
            key,
            AssetMetadataOption.fromJson(value as Map<String, Object?>),
          ),
        ),
      );
  Map<String, Object?> toJson() => {
    'asset_identity': assetIdentity,
    'options': options.map((key, value) => MapEntry(key, value.toJson())),
  };
}

final class AssetMetadataOption {
  const AssetMetadataOption({
    this.name,
    this.symbol,
    this.decimals,
    this.iconUrl,
  });
  final String? name;
  final String? symbol;
  final num? decimals;
  final String? iconUrl;
  factory AssetMetadataOption.fromJson(Map<String, Object?> json) =>
      AssetMetadataOption(
        name: json.containsKey('name') ? json['name'] as String : null,
        symbol: json.containsKey('symbol') ? json['symbol'] as String : null,
        decimals: json.containsKey('decimals') ? json['decimals'] as num : null,
        iconUrl: json.containsKey('icon_url')
            ? json['icon_url'] as String
            : null,
      );
  Map<String, Object?> toJson() => {
    if (name != null) 'name': name!,
    if (symbol != null) 'symbol': symbol!,
    if (decimals != null) 'decimals': decimals!,
    if (iconUrl != null) 'icon_url': iconUrl!,
  };
}

final class AssetParams {
  const AssetParams({required this.assetIdentity});
  final AssetIdentity assetIdentity;
  factory AssetParams.fromJson(Map<String, Object?> json) =>
      AssetParams(assetIdentity: json['asset_identity'] as String);
  Map<String, Object?> toJson() => {'asset_identity': assetIdentity};
}

final class AssetQuoteParams {
  const AssetQuoteParams({required this.assetIdentity, this.displayAsset});
  final AssetIdentity assetIdentity;
  final AssetIdentity? displayAsset;
  factory AssetQuoteParams.fromJson(Map<String, Object?> json) =>
      AssetQuoteParams(
        assetIdentity: json['asset_identity'] as String,
        displayAsset: json.containsKey('display_asset')
            ? json['display_asset'] as String
            : null,
      );
  Map<String, Object?> toJson() => {
    'asset_identity': assetIdentity,
    if (displayAsset != null) 'display_asset': displayAsset!,
  };
}

final class AssetUpdate {
  const AssetUpdate({
    this.assetName,
    this.assetSymbol,
    this.assetDecimals,
    this.assetIconUrl,
  });
  final String? assetName;
  final String? assetSymbol;
  final num? assetDecimals;
  final String? assetIconUrl;
  factory AssetUpdate.fromJson(Map<String, Object?> json) => AssetUpdate(
    assetName: json.containsKey('asset_name')
        ? json['asset_name'] as String
        : null,
    assetSymbol: json.containsKey('asset_symbol')
        ? json['asset_symbol'] as String
        : null,
    assetDecimals: json.containsKey('asset_decimals')
        ? json['asset_decimals'] as num
        : null,
    assetIconUrl: json.containsKey('asset_icon_url')
        ? json['asset_icon_url'] as String
        : null,
  );
  Map<String, Object?> toJson() => {
    if (assetName != null) 'asset_name': assetName!,
    if (assetSymbol != null) 'asset_symbol': assetSymbol!,
    if (assetDecimals != null) 'asset_decimals': assetDecimals!,
    if (assetIconUrl != null) 'asset_icon_url': assetIconUrl!,
  };
}

final class AssetUpdateParams {
  const AssetUpdateParams({required this.assetIdentity, required this.input});
  final AssetIdentity assetIdentity;
  final AssetUpdate input;
  factory AssetUpdateParams.fromJson(Map<String, Object?> json) =>
      AssetUpdateParams(
        assetIdentity: json['asset_identity'] as String,
        input: AssetUpdate.fromJson(json['input'] as Map<String, Object?>),
      );
  Map<String, Object?> toJson() => {
    'asset_identity': assetIdentity,
    'input': input.toJson(),
  };
}

final class DecodeParams {
  const DecodeParams({required this.networkIdentity, required this.input});
  final NetworkIdentity networkIdentity;
  final DecodeTransactionRequest input;
  factory DecodeParams.fromJson(Map<String, Object?> json) => DecodeParams(
    networkIdentity: json['network_identity'] as num,
    input: DecodeTransactionRequest.fromJson(
      json['input'] as Map<String, Object?>,
    ),
  );
  Map<String, Object?> toJson() => {
    'network_identity': networkIdentity,
    'input': input.toJson(),
  };
}

final class DecodeTransactionRequest {
  const DecodeTransactionRequest({
    this.from,
    required this.to,
    this.value,
    this.data,
  });
  final ApiAddress? from;
  final ApiAddress to;
  final ApiU256? value;
  final ApiBytes? data;
  factory DecodeTransactionRequest.fromJson(Map<String, Object?> json) =>
      DecodeTransactionRequest(
        from: json.containsKey('from') ? json['from'] as String : null,
        to: json['to'] as String,
        value: json.containsKey('value') ? json['value'] as String : null,
        data: json.containsKey('data') ? json['data'] as String : null,
      );
  Map<String, Object?> toJson() => {
    if (from != null) 'from': from!,
    'to': to,
    if (value != null) 'value': value!,
    if (data != null) 'data': data!,
  };
}

final class DecodeTransactionResponse {
  const DecodeTransactionResponse({required this.call});
  final DecodedCall call;
  factory DecodeTransactionResponse.fromJson(Map<String, Object?> json) =>
      DecodeTransactionResponse(
        call: DecodedCall.fromJson(json['call'] as Map<String, Object?>),
      );
  Map<String, Object?> toJson() => {'call': call.toJson()};
}

typedef Decoded = Map<String, Object?>;

final class DecodedCall {
  const DecodedCall({
    this.from,
    required this.to,
    required this.value,
    this.operation,
    required this.data,
    this.selector,
    required this.decoded,
    this.subcalls,
  });
  final ApiAddress? from;
  final ApiAddress to;
  final ApiU256 value;
  final String? operation;
  final ApiBytes data;
  final ApiBytes? selector;
  final Decoded decoded;
  final List<DecodedCall>? subcalls;
  factory DecodedCall.fromJson(Map<String, Object?> json) => DecodedCall(
    from: json.containsKey('from') ? json['from'] as String : null,
    to: json['to'] as String,
    value: json['value'] as String,
    operation: json.containsKey('operation')
        ? json['operation'] as String
        : null,
    data: json['data'] as String,
    selector: json.containsKey('selector') ? json['selector'] as String : null,
    decoded: json['decoded'] as Map<String, Object?>,
    subcalls: json.containsKey('subcalls')
        ? decodeRpcList(
            json['subcalls'],
            (value) => DecodedCall.fromJson(value as Map<String, Object?>),
          )
        : null,
  );
  Map<String, Object?> toJson() => {
    if (from != null) 'from': from!,
    'to': to,
    'value': value,
    if (operation != null) 'operation': operation!,
    'data': data,
    if (selector != null) 'selector': selector!,
    'decoded': decoded,
    if (subcalls != null)
      'subcalls': subcalls!.map((value) => value.toJson()).toList(),
  };
}

final class DecodedContract {
  const DecodedContract({required this.address, this.verifiedName, this.proxy});
  final ApiAddress address;
  final String? verifiedName;
  final DecodedProxy? proxy;
  factory DecodedContract.fromJson(Map<String, Object?> json) =>
      DecodedContract(
        address: json['address'] as String,
        verifiedName: json.containsKey('verified_name')
            ? json['verified_name'] as String
            : null,
        proxy: json.containsKey('proxy')
            ? DecodedProxy.fromJson(json['proxy'] as Map<String, Object?>)
            : null,
      );
  Map<String, Object?> toJson() => {
    'address': address,
    if (verifiedName != null) 'verified_name': verifiedName!,
    if (proxy != null) 'proxy': proxy!.toJson(),
  };
}

final class DecodedFunction {
  const DecodedFunction({
    required this.contract,
    required this.selector,
    required this.function,
    required this.signature,
    required this.params,
  });
  final DecodedContract contract;
  final ApiBytes selector;
  final String function;
  final String signature;
  final List<DecodedParam> params;
  factory DecodedFunction.fromJson(Map<String, Object?> json) =>
      DecodedFunction(
        contract: DecodedContract.fromJson(
          json['contract'] as Map<String, Object?>,
        ),
        selector: json['selector'] as String,
        function: json['function'] as String,
        signature: json['signature'] as String,
        params: decodeRpcList(
          json['params'],
          (value) => DecodedParam.fromJson(value as Map<String, Object?>),
        ),
      );
  Map<String, Object?> toJson() => {
    'contract': contract.toJson(),
    'selector': selector,
    'function': function,
    'signature': signature,
    'params': params.map((value) => value.toJson()).toList(),
  };
}

final class DecodedParam {
  const DecodedParam({this.name, required this.ty, required this.value});
  final String? name;
  final String ty;
  final Object? value;
  factory DecodedParam.fromJson(Map<String, Object?> json) => DecodedParam(
    name: json.containsKey('name') ? json['name'] as String : null,
    ty: json['ty'] as String,
    value: json['value'],
  );
  Map<String, Object?> toJson() => {
    if (name != null) 'name': name!,
    'ty': ty,
    'value': value,
  };
}

final class DecodedProxy {
  const DecodedProxy({
    this.proxyType,
    required this.implementation,
    this.implementationName,
  });
  final String? proxyType;
  final ApiAddress implementation;
  final String? implementationName;
  factory DecodedProxy.fromJson(Map<String, Object?> json) => DecodedProxy(
    proxyType: json.containsKey('proxy_type')
        ? json['proxy_type'] as String
        : null,
    implementation: json['implementation'] as String,
    implementationName: json.containsKey('implementation_name')
        ? json['implementation_name'] as String
        : null,
  );
  Map<String, Object?> toJson() => {
    if (proxyType != null) 'proxy_type': proxyType!,
    'implementation': implementation,
    if (implementationName != null) 'implementation_name': implementationName!,
  };
}

final class DeriveMnemonicInput {
  const DeriveMnemonicInput({required this.mnemonic, required this.paths});
  final String mnemonic;
  final List<String> paths;
  factory DeriveMnemonicInput.fromJson(Map<String, Object?> json) =>
      DeriveMnemonicInput(
        mnemonic: json['mnemonic'] as String,
        paths: decodeRpcList(json['paths'], (value) => value as String),
      );
  Map<String, Object?> toJson() => {
    'mnemonic': mnemonic,
    'paths': paths.map((value) => value).toList(),
  };
}

final class DeriveMnemonicParams {
  const DeriveMnemonicParams({required this.input});
  final DeriveMnemonicInput input;
  factory DeriveMnemonicParams.fromJson(Map<String, Object?> json) =>
      DeriveMnemonicParams(
        input: DeriveMnemonicInput.fromJson(
          json['input'] as Map<String, Object?>,
        ),
      );
  Map<String, Object?> toJson() => {'input': input.toJson()};
}

final class DeriveMnemonicResult {
  const DeriveMnemonicResult({required this.path, required this.address});
  final String path;
  final String address;
  factory DeriveMnemonicResult.fromJson(Map<String, Object?> json) =>
      DeriveMnemonicResult(
        path: json['path'] as String,
        address: json['address'] as String,
      );
  Map<String, Object?> toJson() => {'path': path, 'address': address};
}

final class DerivePrivateKeyParams {
  const DerivePrivateKeyParams({required this.input});
  final String input;
  factory DerivePrivateKeyParams.fromJson(Map<String, Object?> json) =>
      DerivePrivateKeyParams(input: json['input'] as String);
  Map<String, Object?> toJson() => {'input': input};
}

final class EOAWallet {
  const EOAWallet({required this.evmAddress});
  final ApiAddress evmAddress;
  factory EOAWallet.fromJson(Map<String, Object?> json) =>
      EOAWallet(evmAddress: json['evm_address'] as String);
  Map<String, Object?> toJson() => {'evm_address': evmAddress};
}

typedef EmptyParams = Map<String, Never>;

final class EndpointCreateParams {
  const EndpointCreateParams({
    required this.networkIdentity,
    required this.input,
  });
  final NetworkIdentity networkIdentity;
  final NetworkEndpoint input;
  factory EndpointCreateParams.fromJson(Map<String, Object?> json) =>
      EndpointCreateParams(
        networkIdentity: json['network_identity'] as num,
        input: NetworkEndpoint.fromJson(json['input'] as Map<String, Object?>),
      );
  Map<String, Object?> toJson() => {
    'network_identity': networkIdentity,
    'input': input.toJson(),
  };
}

final class EndpointParams {
  const EndpointParams({
    required this.networkIdentity,
    required this.endpointIdentity,
  });
  final NetworkIdentity networkIdentity;
  final num endpointIdentity;
  factory EndpointParams.fromJson(Map<String, Object?> json) => EndpointParams(
    networkIdentity: json['network_identity'] as num,
    endpointIdentity: json['endpoint_identity'] as num,
  );
  Map<String, Object?> toJson() => {
    'network_identity': networkIdentity,
    'endpoint_identity': endpointIdentity,
  };
}

final class EndpointUpdateParams {
  const EndpointUpdateParams({
    required this.networkIdentity,
    required this.endpointIdentity,
    required this.input,
  });
  final NetworkIdentity networkIdentity;
  final num endpointIdentity;
  final NetworkEndpointUpdate input;
  factory EndpointUpdateParams.fromJson(Map<String, Object?> json) =>
      EndpointUpdateParams(
        networkIdentity: json['network_identity'] as num,
        endpointIdentity: json['endpoint_identity'] as num,
        input: NetworkEndpointUpdate.fromJson(
          json['input'] as Map<String, Object?>,
        ),
      );
  Map<String, Object?> toJson() => {
    'network_identity': networkIdentity,
    'endpoint_identity': endpointIdentity,
    'input': input.toJson(),
  };
}

typedef Erc4626QuoterConfig = Map<String, Never>;

final class FixedQuoterConfig {
  const FixedQuoterConfig({
    required this.price,
    required this.decimals,
    required this.tokenInDecimals,
    required this.tokenOutDecimals,
  });
  final String price;
  final num decimals;
  final num tokenInDecimals;
  final num tokenOutDecimals;
  factory FixedQuoterConfig.fromJson(Map<String, Object?> json) =>
      FixedQuoterConfig(
        price: json['price'] as String,
        decimals: json['decimals'] as num,
        tokenInDecimals: json['token_in_decimals'] as num,
        tokenOutDecimals: json['token_out_decimals'] as num,
      );
  Map<String, Object?> toJson() => {
    'price': price,
    'decimals': decimals,
    'token_in_decimals': tokenInDecimals,
    'token_out_decimals': tokenOutDecimals,
  };
}

final class GroupCreateParams {
  const GroupCreateParams({required this.input});
  final AccountGroupCreate input;
  factory GroupCreateParams.fromJson(Map<String, Object?> json) =>
      GroupCreateParams(
        input: AccountGroupCreate.fromJson(
          json['input'] as Map<String, Object?>,
        ),
      );
  Map<String, Object?> toJson() => {'input': input.toJson()};
}

typedef GroupIdentity = num;

final class GroupParams {
  const GroupParams({required this.groupIdentity});
  final GroupIdentity groupIdentity;
  factory GroupParams.fromJson(Map<String, Object?> json) =>
      GroupParams(groupIdentity: json['group_identity'] as num);
  Map<String, Object?> toJson() => {'group_identity': groupIdentity};
}

final class GroupUpdateParams {
  const GroupUpdateParams({required this.groupIdentity, required this.input});
  final GroupIdentity groupIdentity;
  final AccountGroupUpdate input;
  factory GroupUpdateParams.fromJson(Map<String, Object?> json) =>
      GroupUpdateParams(
        groupIdentity: json['group_identity'] as num,
        input: AccountGroupUpdate.fromJson(
          json['input'] as Map<String, Object?>,
        ),
      );
  Map<String, Object?> toJson() => {
    'group_identity': groupIdentity,
    'input': input.toJson(),
  };
}

enum JsonRpcVersion {
  value20('2.0');

  const JsonRpcVersion(this.wireValue);
  final String wireValue;
}

extension JsonRpcVersionCodec on JsonRpcVersion {
  static JsonRpcVersion fromJson(Object? value) =>
      JsonRpcVersion.values.singleWhere((entry) => entry.wireValue == value);
}

final class LayoutUpdateParams {
  const LayoutUpdateParams({required this.input});
  final AccountLayoutUpdate input;
  factory LayoutUpdateParams.fromJson(Map<String, Object?> json) =>
      LayoutUpdateParams(
        input: AccountLayoutUpdate.fromJson(
          json['input'] as Map<String, Object?>,
        ),
      );
  Map<String, Object?> toJson() => {'input': input.toJson()};
}

final class Network {
  const Network({
    required this.networkIdentity,
    required this.networkName,
    this.networkIconUrl,
  });
  final NetworkIdentity networkIdentity;
  final String networkName;
  final String? networkIconUrl;
  factory Network.fromJson(Map<String, Object?> json) => Network(
    networkIdentity: json['network_identity'] as num,
    networkName: json['network_name'] as String,
    networkIconUrl: json.containsKey('network_icon_url')
        ? json['network_icon_url'] as String
        : null,
  );
  Map<String, Object?> toJson() => {
    'network_identity': networkIdentity,
    'network_name': networkName,
    if (networkIconUrl != null) 'network_icon_url': networkIconUrl!,
  };
}

final class NetworkCreateParams {
  const NetworkCreateParams({required this.input});
  final Network input;
  factory NetworkCreateParams.fromJson(Map<String, Object?> json) =>
      NetworkCreateParams(
        input: Network.fromJson(json['input'] as Map<String, Object?>),
      );
  Map<String, Object?> toJson() => {'input': input.toJson()};
}

final class NetworkEndpoint {
  const NetworkEndpoint({
    required this.endpointIdentity,
    this.endpointLabel,
    required this.endpointType,
    required this.endpointUrl,
    required this.endpointDisabled,
    required this.networkIdentity,
  });
  final num endpointIdentity;
  final String? endpointLabel;
  final String endpointType;
  final String endpointUrl;
  final bool endpointDisabled;
  final NetworkIdentity networkIdentity;
  factory NetworkEndpoint.fromJson(Map<String, Object?> json) =>
      NetworkEndpoint(
        endpointIdentity: json['endpoint_identity'] as num,
        endpointLabel: json.containsKey('endpoint_label')
            ? json['endpoint_label'] as String
            : null,
        endpointType: json['endpoint_type'] as String,
        endpointUrl: json['endpoint_url'] as String,
        endpointDisabled: json['endpoint_disabled'] as bool,
        networkIdentity: json['network_identity'] as num,
      );
  Map<String, Object?> toJson() => {
    'endpoint_identity': endpointIdentity,
    if (endpointLabel != null) 'endpoint_label': endpointLabel!,
    'endpoint_type': endpointType,
    'endpoint_url': endpointUrl,
    'endpoint_disabled': endpointDisabled,
    'network_identity': networkIdentity,
  };
}

final class NetworkEndpointUpdate {
  const NetworkEndpointUpdate({
    this.endpointLabel,
    this.endpointType,
    this.endpointUrl,
    this.endpointDisabled,
  });
  final String? endpointLabel;
  final String? endpointType;
  final String? endpointUrl;
  final bool? endpointDisabled;
  factory NetworkEndpointUpdate.fromJson(Map<String, Object?> json) =>
      NetworkEndpointUpdate(
        endpointLabel: json.containsKey('endpoint_label')
            ? json['endpoint_label'] as String
            : null,
        endpointType: json.containsKey('endpoint_type')
            ? json['endpoint_type'] as String
            : null,
        endpointUrl: json.containsKey('endpoint_url')
            ? json['endpoint_url'] as String
            : null,
        endpointDisabled: json.containsKey('endpoint_disabled')
            ? json['endpoint_disabled'] as bool
            : null,
      );
  Map<String, Object?> toJson() => {
    if (endpointLabel != null) 'endpoint_label': endpointLabel!,
    if (endpointType != null) 'endpoint_type': endpointType!,
    if (endpointUrl != null) 'endpoint_url': endpointUrl!,
    if (endpointDisabled != null) 'endpoint_disabled': endpointDisabled!,
  };
}

typedef NetworkIdentity = num;

final class NetworkMetadataDiscovery {
  const NetworkMetadataDiscovery({
    required this.networkIdentity,
    required this.options,
  });
  final NetworkIdentity networkIdentity;
  final Map<String, NetworkMetadataOption> options;
  factory NetworkMetadataDiscovery.fromJson(Map<String, Object?> json) =>
      NetworkMetadataDiscovery(
        networkIdentity: json['network_identity'] as num,
        options: (json['options'] as Map<String, Object?>).map(
          (key, value) => MapEntry(
            key,
            NetworkMetadataOption.fromJson(value as Map<String, Object?>),
          ),
        ),
      );
  Map<String, Object?> toJson() => {
    'network_identity': networkIdentity,
    'options': options.map((key, value) => MapEntry(key, value.toJson())),
  };
}

final class NetworkMetadataOption {
  const NetworkMetadataOption({this.iconUrl});
  final String? iconUrl;
  factory NetworkMetadataOption.fromJson(Map<String, Object?> json) =>
      NetworkMetadataOption(
        iconUrl: json.containsKey('icon_url')
            ? json['icon_url'] as String
            : null,
      );
  Map<String, Object?> toJson() => {if (iconUrl != null) 'icon_url': iconUrl!};
}

final class NetworkParams {
  const NetworkParams({required this.networkIdentity});
  final NetworkIdentity networkIdentity;
  factory NetworkParams.fromJson(Map<String, Object?> json) =>
      NetworkParams(networkIdentity: json['network_identity'] as num);
  Map<String, Object?> toJson() => {'network_identity': networkIdentity};
}

final class NetworkUpdate {
  const NetworkUpdate({this.networkName, this.networkIconUrl});
  final String? networkName;
  final String? networkIconUrl;
  factory NetworkUpdate.fromJson(Map<String, Object?> json) => NetworkUpdate(
    networkName: json.containsKey('network_name')
        ? json['network_name'] as String
        : null,
    networkIconUrl: json.containsKey('network_icon_url')
        ? json['network_icon_url'] as String
        : null,
  );
  Map<String, Object?> toJson() => {
    if (networkName != null) 'network_name': networkName!,
    if (networkIconUrl != null) 'network_icon_url': networkIconUrl!,
  };
}

final class NetworkUpdateParams {
  const NetworkUpdateParams({
    required this.networkIdentity,
    required this.input,
  });
  final NetworkIdentity networkIdentity;
  final NetworkUpdate input;
  factory NetworkUpdateParams.fromJson(Map<String, Object?> json) =>
      NetworkUpdateParams(
        networkIdentity: json['network_identity'] as num,
        input: NetworkUpdate.fromJson(json['input'] as Map<String, Object?>),
      );
  Map<String, Object?> toJson() => {
    'network_identity': networkIdentity,
    'input': input.toJson(),
  };
}

final class Quoter {
  const Quoter({
    required this.quoterIdentity,
    required this.quoterName,
    required this.tokenA,
    required this.tokenB,
    required this.config,
    required this.enabled,
    required this.watch,
  });
  final String quoterIdentity;
  final String quoterName;
  final AssetIdentity tokenA;
  final AssetIdentity tokenB;
  final QuoterConfig config;
  final bool enabled;
  final bool watch;
  factory Quoter.fromJson(Map<String, Object?> json) => Quoter(
    quoterIdentity: json['quoter_identity'] as String,
    quoterName: json['quoter_name'] as String,
    tokenA: json['token_a'] as String,
    tokenB: json['token_b'] as String,
    config: json['config'] as Map<String, Object?>,
    enabled: json['enabled'] as bool,
    watch: json['watch'] as bool,
  );
  Map<String, Object?> toJson() => {
    'quoter_identity': quoterIdentity,
    'quoter_name': quoterName,
    'token_a': tokenA,
    'token_b': tokenB,
    'config': config,
    'enabled': enabled,
    'watch': watch,
  };
}

typedef QuoterConfig = Map<String, Object?>;

final class QuoterCreate {
  const QuoterCreate({
    required this.quoterName,
    required this.tokenA,
    required this.tokenB,
    required this.config,
    required this.enabled,
    required this.watch,
  });
  final String quoterName;
  final AssetIdentity tokenA;
  final AssetIdentity tokenB;
  final QuoterConfig config;
  final bool enabled;
  final bool watch;
  factory QuoterCreate.fromJson(Map<String, Object?> json) => QuoterCreate(
    quoterName: json['quoter_name'] as String,
    tokenA: json['token_a'] as String,
    tokenB: json['token_b'] as String,
    config: json['config'] as Map<String, Object?>,
    enabled: json['enabled'] as bool,
    watch: json['watch'] as bool,
  );
  Map<String, Object?> toJson() => {
    'quoter_name': quoterName,
    'token_a': tokenA,
    'token_b': tokenB,
    'config': config,
    'enabled': enabled,
    'watch': watch,
  };
}

final class QuoterCreateParams {
  const QuoterCreateParams({required this.input});
  final QuoterCreate input;
  factory QuoterCreateParams.fromJson(Map<String, Object?> json) =>
      QuoterCreateParams(
        input: QuoterCreate.fromJson(json['input'] as Map<String, Object?>),
      );
  Map<String, Object?> toJson() => {'input': input.toJson()};
}

final class QuoterDiscoverParams {
  const QuoterDiscoverParams({required this.input});
  final QuoterDiscovery input;
  factory QuoterDiscoverParams.fromJson(Map<String, Object?> json) =>
      QuoterDiscoverParams(
        input: QuoterDiscovery.fromJson(json['input'] as Map<String, Object?>),
      );
  Map<String, Object?> toJson() => {'input': input.toJson()};
}

final class QuoterDiscovery {
  const QuoterDiscovery({required this.tokenA, this.tokenB});
  final AssetIdentity tokenA;
  final AssetIdentity? tokenB;
  factory QuoterDiscovery.fromJson(Map<String, Object?> json) =>
      QuoterDiscovery(
        tokenA: json['token_a'] as String,
        tokenB: json.containsKey('token_b') ? json['token_b'] as String : null,
      );
  Map<String, Object?> toJson() => {
    'token_a': tokenA,
    if (tokenB != null) 'token_b': tokenB!,
  };
}

final class QuoterDiscoveryResponse {
  const QuoterDiscoveryResponse({this.erc4626, this.uniswapV2, this.uniswapV3});
  final AssetIdentity? erc4626;
  final UniswapV2Pair? uniswapV2;
  final List<UniswapV3Pool>? uniswapV3;
  factory QuoterDiscoveryResponse.fromJson(Map<String, Object?> json) =>
      QuoterDiscoveryResponse(
        erc4626: json.containsKey('erc4626') ? json['erc4626'] as String : null,
        uniswapV2: json.containsKey('uniswap_v2')
            ? UniswapV2Pair.fromJson(json['uniswap_v2'] as Map<String, Object?>)
            : null,
        uniswapV3: json.containsKey('uniswap_v3')
            ? decodeRpcList(
                json['uniswap_v3'],
                (value) =>
                    UniswapV3Pool.fromJson(value as Map<String, Object?>),
              )
            : null,
      );
  Map<String, Object?> toJson() => {
    if (erc4626 != null) 'erc4626': erc4626!,
    if (uniswapV2 != null) 'uniswap_v2': uniswapV2!.toJson(),
    if (uniswapV3 != null)
      'uniswap_v3': uniswapV3!.map((value) => value.toJson()).toList(),
  };
}

final class QuoterParams {
  const QuoterParams({required this.quoterIdentity});
  final String quoterIdentity;
  factory QuoterParams.fromJson(Map<String, Object?> json) =>
      QuoterParams(quoterIdentity: json['quoter_identity'] as String);
  Map<String, Object?> toJson() => {'quoter_identity': quoterIdentity};
}

final class QuoterUpdate {
  const QuoterUpdate({
    this.quoterName,
    this.tokenA,
    this.tokenB,
    this.config,
    this.enabled,
    this.watch,
  });
  final String? quoterName;
  final AssetIdentity? tokenA;
  final AssetIdentity? tokenB;
  final QuoterConfig? config;
  final bool? enabled;
  final bool? watch;
  factory QuoterUpdate.fromJson(Map<String, Object?> json) => QuoterUpdate(
    quoterName: json.containsKey('quoter_name')
        ? json['quoter_name'] as String
        : null,
    tokenA: json.containsKey('token_a') ? json['token_a'] as String : null,
    tokenB: json.containsKey('token_b') ? json['token_b'] as String : null,
    config: json.containsKey('config')
        ? json['config'] as Map<String, Object?>
        : null,
    enabled: json.containsKey('enabled') ? json['enabled'] as bool : null,
    watch: json.containsKey('watch') ? json['watch'] as bool : null,
  );
  Map<String, Object?> toJson() => {
    if (quoterName != null) 'quoter_name': quoterName!,
    if (tokenA != null) 'token_a': tokenA!,
    if (tokenB != null) 'token_b': tokenB!,
    if (config != null) 'config': config!,
    if (enabled != null) 'enabled': enabled!,
    if (watch != null) 'watch': watch!,
  };
}

final class QuoterUpdateParams {
  const QuoterUpdateParams({required this.quoterIdentity, required this.input});
  final String quoterIdentity;
  final QuoterUpdate input;
  factory QuoterUpdateParams.fromJson(Map<String, Object?> json) =>
      QuoterUpdateParams(
        quoterIdentity: json['quoter_identity'] as String,
        input: QuoterUpdate.fromJson(json['input'] as Map<String, Object?>),
      );
  Map<String, Object?> toJson() => {
    'quoter_identity': quoterIdentity,
    'input': input.toJson(),
  };
}

final class RailgunWallet {
  const RailgunWallet({required this.railgunAddress});
  final String railgunAddress;
  factory RailgunWallet.fromJson(Map<String, Object?> json) =>
      RailgunWallet(railgunAddress: json['railgun_address'] as String);
  Map<String, Object?> toJson() => {'railgun_address': railgunAddress};
}

final class RawCall {
  const RawCall({required this.data});
  final ApiBytes data;
  factory RawCall.fromJson(Map<String, Object?> json) =>
      RawCall(data: json['data'] as String);
  Map<String, Object?> toJson() => {'data': data};
}

final class RpcCallSample {
  const RpcCallSample({
    required this.timestamp,
    required this.methods,
    required this.requestCount,
    required this.durationMs,
    required this.success,
    this.error,
  });
  final num timestamp;
  final List<String> methods;
  final num requestCount;
  final num durationMs;
  final bool success;
  final String? error;
  factory RpcCallSample.fromJson(Map<String, Object?> json) => RpcCallSample(
    timestamp: json['timestamp'] as num,
    methods: decodeRpcList(json['methods'], (value) => value as String),
    requestCount: json['request_count'] as num,
    durationMs: json['duration_ms'] as num,
    success: json['success'] as bool,
    error: json.containsKey('error') ? json['error'] as String : null,
  );
  Map<String, Object?> toJson() => {
    'timestamp': timestamp,
    'methods': methods.map((value) => value).toList(),
    'request_count': requestCount,
    'duration_ms': durationMs,
    'success': success,
    if (error != null) 'error': error!,
  };
}

final class RpcEndpointStats {
  const RpcEndpointStats({
    required this.inFlight,
    required this.queued,
    required this.maxInFlight,
    required this.totalRequests,
    required this.totalErrors,
    required this.totalRateLimited,
    required this.averageDurationMs,
    required this.connectionAttempts,
    required this.connectionSuccesses,
    required this.connectionFailures,
    this.lastRequestAt,
    this.lastErrorAt,
    this.lastConnectedAt,
    this.lastError,
    required this.methods,
    required this.recent,
  });
  final num inFlight;
  final num queued;
  final num maxInFlight;
  final num totalRequests;
  final num totalErrors;
  final num totalRateLimited;
  final num averageDurationMs;
  final num connectionAttempts;
  final num connectionSuccesses;
  final num connectionFailures;
  final num? lastRequestAt;
  final num? lastErrorAt;
  final num? lastConnectedAt;
  final String? lastError;
  final List<RpcMethodStats> methods;
  final List<RpcCallSample> recent;
  factory RpcEndpointStats.fromJson(Map<String, Object?> json) =>
      RpcEndpointStats(
        inFlight: json['in_flight'] as num,
        queued: json['queued'] as num,
        maxInFlight: json['max_in_flight'] as num,
        totalRequests: json['total_requests'] as num,
        totalErrors: json['total_errors'] as num,
        totalRateLimited: json['total_rate_limited'] as num,
        averageDurationMs: json['average_duration_ms'] as num,
        connectionAttempts: json['connection_attempts'] as num,
        connectionSuccesses: json['connection_successes'] as num,
        connectionFailures: json['connection_failures'] as num,
        lastRequestAt: json.containsKey('last_request_at')
            ? json['last_request_at'] as num
            : null,
        lastErrorAt: json.containsKey('last_error_at')
            ? json['last_error_at'] as num
            : null,
        lastConnectedAt: json.containsKey('last_connected_at')
            ? json['last_connected_at'] as num
            : null,
        lastError: json.containsKey('last_error')
            ? json['last_error'] as String
            : null,
        methods: decodeRpcList(
          json['methods'],
          (value) => RpcMethodStats.fromJson(value as Map<String, Object?>),
        ),
        recent: decodeRpcList(
          json['recent'],
          (value) => RpcCallSample.fromJson(value as Map<String, Object?>),
        ),
      );
  Map<String, Object?> toJson() => {
    'in_flight': inFlight,
    'queued': queued,
    'max_in_flight': maxInFlight,
    'total_requests': totalRequests,
    'total_errors': totalErrors,
    'total_rate_limited': totalRateLimited,
    'average_duration_ms': averageDurationMs,
    'connection_attempts': connectionAttempts,
    'connection_successes': connectionSuccesses,
    'connection_failures': connectionFailures,
    if (lastRequestAt != null) 'last_request_at': lastRequestAt!,
    if (lastErrorAt != null) 'last_error_at': lastErrorAt!,
    if (lastConnectedAt != null) 'last_connected_at': lastConnectedAt!,
    if (lastError != null) 'last_error': lastError!,
    'methods': methods.map((value) => value.toJson()).toList(),
    'recent': recent.map((value) => value.toJson()).toList(),
  };
}

final class RpcErrorData {
  const RpcErrorData({required this.kind, required this.message});
  final RpcErrorKind kind;
  final String message;
  factory RpcErrorData.fromJson(Map<String, Object?> json) => RpcErrorData(
    kind: RpcErrorKindCodec.fromJson(json['kind']),
    message: json['message'] as String,
  );
  Map<String, Object?> toJson() => {'kind': kind.wireValue, 'message': message};
}

final class RpcErrorEnvelope {
  const RpcErrorEnvelope({
    required this.jsonrpc,
    required this.id,
    required this.error,
  });
  final JsonRpcVersion jsonrpc;
  final RpcIdentity id;
  final RpcErrorObject error;
  factory RpcErrorEnvelope.fromJson(Map<String, Object?> json) =>
      RpcErrorEnvelope(
        jsonrpc: JsonRpcVersionCodec.fromJson(json['jsonrpc']),
        id: json['id'] as Map<String, Object?>,
        error: RpcErrorObject.fromJson(json['error'] as Map<String, Object?>),
      );
  Map<String, Object?> toJson() => {
    'jsonrpc': jsonrpc.wireValue,
    'id': id,
    'error': error.toJson(),
  };
}

enum RpcErrorKind {
  invalidInput('invalid_input'),
  notFound('not_found'),
  conflict('conflict'),
  unavailable('unavailable'),
  internal('internal');

  const RpcErrorKind(this.wireValue);
  final String wireValue;
}

extension RpcErrorKindCodec on RpcErrorKind {
  static RpcErrorKind fromJson(Object? value) =>
      RpcErrorKind.values.singleWhere((entry) => entry.wireValue == value);
}

final class RpcErrorObject {
  const RpcErrorObject({required this.code, required this.message, this.data});
  final num code;
  final String message;
  final RpcErrorData? data;
  factory RpcErrorObject.fromJson(Map<String, Object?> json) => RpcErrorObject(
    code: json['code'] as num,
    message: json['message'] as String,
    data: json.containsKey('data')
        ? RpcErrorData.fromJson(json['data'] as Map<String, Object?>)
        : null,
  );
  Map<String, Object?> toJson() => {
    'code': code,
    'message': message,
    if (data != null) 'data': data!.toJson(),
  };
}

typedef RpcIdentity = Map<String, Object?>;

final class RpcMethodStats {
  const RpcMethodStats({
    required this.method,
    required this.total,
    required this.errors,
  });
  final String method;
  final num total;
  final num errors;
  factory RpcMethodStats.fromJson(Map<String, Object?> json) => RpcMethodStats(
    method: json['method'] as String,
    total: json['total'] as num,
    errors: json['errors'] as num,
  );
  Map<String, Object?> toJson() => {
    'method': method,
    'total': total,
    'errors': errors,
  };
}

final class RpcPoolEndpointStats {
  const RpcPoolEndpointStats({
    required this.endpointIdentity,
    required this.status,
    required this.inFlight,
    required this.queued,
    required this.totalRequests,
    required this.totalErrors,
  });
  final num endpointIdentity;
  final String status;
  final num inFlight;
  final num queued;
  final num totalRequests;
  final num totalErrors;
  factory RpcPoolEndpointStats.fromJson(Map<String, Object?> json) =>
      RpcPoolEndpointStats(
        endpointIdentity: json['endpoint_identity'] as num,
        status: json['status'] as String,
        inFlight: json['in_flight'] as num,
        queued: json['queued'] as num,
        totalRequests: json['total_requests'] as num,
        totalErrors: json['total_errors'] as num,
      );
  Map<String, Object?> toJson() => {
    'endpoint_identity': endpointIdentity,
    'status': status,
    'in_flight': inFlight,
    'queued': queued,
    'total_requests': totalRequests,
    'total_errors': totalErrors,
  };
}

final class RpcPoolStats {
  const RpcPoolStats({
    required this.networkIdentity,
    required this.endpointCount,
    required this.aliveCount,
    required this.deadCount,
    required this.disabledCount,
    required this.inFlight,
    required this.queued,
    required this.totalRequests,
    required this.totalErrors,
    required this.endpoints,
  });
  final NetworkIdentity networkIdentity;
  final num endpointCount;
  final num aliveCount;
  final num deadCount;
  final num disabledCount;
  final num inFlight;
  final num queued;
  final num totalRequests;
  final num totalErrors;
  final List<RpcPoolEndpointStats> endpoints;
  factory RpcPoolStats.fromJson(Map<String, Object?> json) => RpcPoolStats(
    networkIdentity: json['network_identity'] as num,
    endpointCount: json['endpoint_count'] as num,
    aliveCount: json['alive_count'] as num,
    deadCount: json['dead_count'] as num,
    disabledCount: json['disabled_count'] as num,
    inFlight: json['in_flight'] as num,
    queued: json['queued'] as num,
    totalRequests: json['total_requests'] as num,
    totalErrors: json['total_errors'] as num,
    endpoints: decodeRpcList(
      json['endpoints'],
      (value) => RpcPoolEndpointStats.fromJson(value as Map<String, Object?>),
    ),
  );
  Map<String, Object?> toJson() => {
    'network_identity': networkIdentity,
    'endpoint_count': endpointCount,
    'alive_count': aliveCount,
    'dead_count': deadCount,
    'disabled_count': disabledCount,
    'in_flight': inFlight,
    'queued': queued,
    'total_requests': totalRequests,
    'total_errors': totalErrors,
    'endpoints': endpoints.map((value) => value.toJson()).toList(),
  };
}

final class RpcRequestEnvelope {
  const RpcRequestEnvelope({
    required this.jsonrpc,
    this.id,
    required this.method,
    required this.params,
  });
  final JsonRpcVersion jsonrpc;
  final RpcIdentity? id;
  final String method;
  final Map<String, Object?> params;
  factory RpcRequestEnvelope.fromJson(Map<String, Object?> json) =>
      RpcRequestEnvelope(
        jsonrpc: JsonRpcVersionCodec.fromJson(json['jsonrpc']),
        id: json.containsKey('id') ? json['id'] as Map<String, Object?> : null,
        method: json['method'] as String,
        params: (json['params'] as Map<String, Object?>).map(
          (key, value) => MapEntry(key, value),
        ),
      );
  Map<String, Object?> toJson() => {
    'jsonrpc': jsonrpc.wireValue,
    if (id != null) 'id': id!,
    'method': method,
    'params': params.map((key, value) => MapEntry(key, value)),
  };
}

typedef RpcResponseEnvelope = Map<String, Object?>;

typedef RpcStatus = Map<String, Object?>;

final class RpcStatusAlive {
  const RpcStatusAlive({
    required this.blockNumber,
    required this.networkIdentity,
    required this.timestamp,
    required this.rpc,
  });
  final num blockNumber;
  final NetworkIdentity networkIdentity;
  final num timestamp;
  final RpcEndpointStats rpc;
  factory RpcStatusAlive.fromJson(Map<String, Object?> json) => RpcStatusAlive(
    blockNumber: json['block_number'] as num,
    networkIdentity: json['network_identity'] as num,
    timestamp: json['timestamp'] as num,
    rpc: RpcEndpointStats.fromJson(json['rpc'] as Map<String, Object?>),
  );
  Map<String, Object?> toJson() => {
    'block_number': blockNumber,
    'network_identity': networkIdentity,
    'timestamp': timestamp,
    'rpc': rpc.toJson(),
  };
}

final class RpcStatusDead {
  const RpcStatusDead({required this.error, required this.rpc});
  final String error;
  final RpcEndpointStats rpc;
  factory RpcStatusDead.fromJson(Map<String, Object?> json) => RpcStatusDead(
    error: json['error'] as String,
    rpc: RpcEndpointStats.fromJson(json['rpc'] as Map<String, Object?>),
  );
  Map<String, Object?> toJson() => {'error': error, 'rpc': rpc.toJson()};
}

final class RpcStatusDisabled {
  const RpcStatusDisabled({required this.rpc});
  final RpcEndpointStats rpc;
  factory RpcStatusDisabled.fromJson(Map<String, Object?> json) =>
      RpcStatusDisabled(
        rpc: RpcEndpointStats.fromJson(json['rpc'] as Map<String, Object?>),
      );
  Map<String, Object?> toJson() => {'rpc': rpc.toJson()};
}

final class RpcSuccessEnvelope {
  const RpcSuccessEnvelope({
    required this.jsonrpc,
    required this.id,
    required this.result,
  });
  final JsonRpcVersion jsonrpc;
  final RpcIdentity id;
  final Object? result;
  factory RpcSuccessEnvelope.fromJson(Map<String, Object?> json) =>
      RpcSuccessEnvelope(
        jsonrpc: JsonRpcVersionCodec.fromJson(json['jsonrpc']),
        id: json['id'] as Map<String, Object?>,
        result: json['result'],
      );
  Map<String, Object?> toJson() => {
    'jsonrpc': jsonrpc.wireValue,
    'id': id,
    'result': result,
  };
}

final class SafeWallet {
  const SafeWallet({required this.evmAddress});
  final ApiAddress evmAddress;
  factory SafeWallet.fromJson(Map<String, Object?> json) =>
      SafeWallet(evmAddress: json['evm_address'] as String);
  Map<String, Object?> toJson() => {'evm_address': evmAddress};
}

final class SafeWalletTxExtra {
  const SafeWalletTxExtra({
    this.nonce,
    this.executionDate,
    this.safeTxHash,
    this.proposer,
    this.executor,
    this.isSuccessful,
    this.isExecuted,
    this.origin,
    required this.extra,
  });
  final num? nonce;
  final String? executionDate;
  final ApiBytes? safeTxHash;
  final ApiAddress? proposer;
  final ApiAddress? executor;
  final bool? isSuccessful;
  final bool? isExecuted;
  final String? origin;
  final Object? extra;
  factory SafeWalletTxExtra.fromJson(
    Map<String, Object?> json,
  ) => SafeWalletTxExtra(
    nonce: json.containsKey('nonce') ? json['nonce'] as num : null,
    executionDate: json.containsKey('execution_date')
        ? json['execution_date'] as String
        : null,
    safeTxHash: json.containsKey('safe_tx_hash')
        ? json['safe_tx_hash'] as String
        : null,
    proposer: json.containsKey('proposer') ? json['proposer'] as String : null,
    executor: json.containsKey('executor') ? json['executor'] as String : null,
    isSuccessful: json.containsKey('is_successful')
        ? json['is_successful'] as bool
        : null,
    isExecuted: json.containsKey('is_executed')
        ? json['is_executed'] as bool
        : null,
    origin: json.containsKey('origin') ? json['origin'] as String : null,
    extra: json['extra'],
  );
  Map<String, Object?> toJson() => {
    if (nonce != null) 'nonce': nonce!,
    if (executionDate != null) 'execution_date': executionDate!,
    if (safeTxHash != null) 'safe_tx_hash': safeTxHash!,
    if (proposer != null) 'proposer': proposer!,
    if (executor != null) 'executor': executor!,
    if (isSuccessful != null) 'is_successful': isSuccessful!,
    if (isExecuted != null) 'is_executed': isExecuted!,
    if (origin != null) 'origin': origin!,
    'extra': extra,
  };
}

final class SignatureFallback {
  const SignatureFallback({
    this.contract,
    required this.selector,
    required this.candidates,
  });
  final DecodedContract? contract;
  final ApiBytes selector;
  final List<String> candidates;
  factory SignatureFallback.fromJson(Map<String, Object?> json) =>
      SignatureFallback(
        contract: json.containsKey('contract')
            ? DecodedContract.fromJson(json['contract'] as Map<String, Object?>)
            : null,
        selector: json['selector'] as String,
        candidates: decodeRpcList(
          json['candidates'],
          (value) => value as String,
        ),
      );
  Map<String, Object?> toJson() => {
    if (contract != null) 'contract': contract!.toJson(),
    'selector': selector,
    'candidates': candidates.map((value) => value).toList(),
  };
}

final class SimulateParams {
  const SimulateParams({required this.networkIdentity, required this.input});
  final NetworkIdentity networkIdentity;
  final SimulateTransactionRequest input;
  factory SimulateParams.fromJson(Map<String, Object?> json) => SimulateParams(
    networkIdentity: json['network_identity'] as num,
    input: SimulateTransactionRequest.fromJson(
      json['input'] as Map<String, Object?>,
    ),
  );
  Map<String, Object?> toJson() => {
    'network_identity': networkIdentity,
    'input': input.toJson(),
  };
}

final class SimulateTransactionRequest {
  const SimulateTransactionRequest({
    this.from,
    required this.to,
    this.value,
    this.data,
  });
  final ApiAddress? from;
  final ApiAddress to;
  final ApiU256? value;
  final ApiBytes? data;
  factory SimulateTransactionRequest.fromJson(Map<String, Object?> json) =>
      SimulateTransactionRequest(
        from: json.containsKey('from') ? json['from'] as String : null,
        to: json['to'] as String,
        value: json.containsKey('value') ? json['value'] as String : null,
        data: json.containsKey('data') ? json['data'] as String : null,
      );
  Map<String, Object?> toJson() => {
    if (from != null) 'from': from!,
    'to': to,
    if (value != null) 'value': value!,
    if (data != null) 'data': data!,
  };
}

final class SimulateTransactionResponse {
  const SimulateTransactionResponse({required this.call});
  final DecodedCall call;
  factory SimulateTransactionResponse.fromJson(Map<String, Object?> json) =>
      SimulateTransactionResponse(
        call: DecodedCall.fromJson(json['call'] as Map<String, Object?>),
      );
  Map<String, Object?> toJson() => {'call': call.toJson()};
}

final class Tx {
  const Tx({
    required this.networkIdentity,
    this.txHash,
    this.from,
    this.to,
    this.data,
    required this.value,
    this.decoded,
    required this.extra,
  });
  final NetworkIdentity networkIdentity;
  final ApiBytes? txHash;
  final ApiAddress? from;
  final ApiAddress? to;
  final ApiBytes? data;
  final ApiU256 value;
  final DecodedCall? decoded;
  final TxExtra extra;
  factory Tx.fromJson(Map<String, Object?> json) => Tx(
    networkIdentity: json['network_identity'] as num,
    txHash: json.containsKey('tx_hash') ? json['tx_hash'] as String : null,
    from: json.containsKey('from') ? json['from'] as String : null,
    to: json.containsKey('to') ? json['to'] as String : null,
    data: json.containsKey('data') ? json['data'] as String : null,
    value: json['value'] as String,
    decoded: json.containsKey('decoded')
        ? DecodedCall.fromJson(json['decoded'] as Map<String, Object?>)
        : null,
    extra: TxExtra.fromJson(json['extra'] as Map<String, Object?>),
  );
  Map<String, Object?> toJson() => {
    'network_identity': networkIdentity,
    if (txHash != null) 'tx_hash': txHash!,
    if (from != null) 'from': from!,
    if (to != null) 'to': to!,
    if (data != null) 'data': data!,
    'value': value,
    if (decoded != null) 'decoded': decoded!.toJson(),
    'extra': extra.toJson(),
  };
}

final class TxExtra {
  const TxExtra({this.safeWallet});
  final SafeWalletTxExtra? safeWallet;
  factory TxExtra.fromJson(Map<String, Object?> json) => TxExtra(
    safeWallet: json.containsKey('safe_wallet')
        ? SafeWalletTxExtra.fromJson(
            json['safe_wallet'] as Map<String, Object?>,
          )
        : null,
  );
  Map<String, Object?> toJson() => {
    if (safeWallet != null) 'safe_wallet': safeWallet!.toJson(),
  };
}

final class UniswapV2Pair {
  const UniswapV2Pair({
    required this.pairAddress,
    this.reserve0,
    this.reserve1,
  });
  final String pairAddress;
  final String? reserve0;
  final String? reserve1;
  factory UniswapV2Pair.fromJson(Map<String, Object?> json) => UniswapV2Pair(
    pairAddress: json['pair_address'] as String,
    reserve0: json.containsKey('reserve_0')
        ? json['reserve_0'] as String
        : null,
    reserve1: json.containsKey('reserve_1')
        ? json['reserve_1'] as String
        : null,
  );
  Map<String, Object?> toJson() => {
    'pair_address': pairAddress,
    if (reserve0 != null) 'reserve_0': reserve0!,
    if (reserve1 != null) 'reserve_1': reserve1!,
  };
}

final class UniswapV2QuoterConfig {
  const UniswapV2QuoterConfig({required this.pairAddress});
  final String pairAddress;
  factory UniswapV2QuoterConfig.fromJson(Map<String, Object?> json) =>
      UniswapV2QuoterConfig(pairAddress: json['pair_address'] as String);
  Map<String, Object?> toJson() => {'pair_address': pairAddress};
}

final class UniswapV3Pool {
  const UniswapV3Pool({
    required this.poolAddress,
    required this.fee,
    this.reserve0,
    this.reserve1,
  });
  final String poolAddress;
  final num fee;
  final String? reserve0;
  final String? reserve1;
  factory UniswapV3Pool.fromJson(Map<String, Object?> json) => UniswapV3Pool(
    poolAddress: json['pool_address'] as String,
    fee: json['fee'] as num,
    reserve0: json.containsKey('reserve_0')
        ? json['reserve_0'] as String
        : null,
    reserve1: json.containsKey('reserve_1')
        ? json['reserve_1'] as String
        : null,
  );
  Map<String, Object?> toJson() => {
    'pool_address': poolAddress,
    'fee': fee,
    if (reserve0 != null) 'reserve_0': reserve0!,
    if (reserve1 != null) 'reserve_1': reserve1!,
  };
}

final class UniswapV3QuoterConfig {
  const UniswapV3QuoterConfig({required this.poolAddress});
  final String poolAddress;
  factory UniswapV3QuoterConfig.fromJson(Map<String, Object?> json) =>
      UniswapV3QuoterConfig(poolAddress: json['pool_address'] as String);
  Map<String, Object?> toJson() => {'pool_address': poolAddress};
}

enum VendorFlag {
  avaraAssetIcons('avara_asset_icons'),
  zerionAssetIcons('zerion_asset_icons'),
  smoldappAssetIcons('smoldapp_asset_icons'),
  smoldappNetworkIcons('smoldapp_network_icons'),
  safewalletNetworkIcons('safewallet_network_icons'),
  safewalletAssetIcons('safewallet_asset_icons'),
  safewalletTransactionsApi('safewallet_transactions_api'),
  etherscanAssetIcons('etherscan_asset_icons'),
  etherscanLinkTxHash('etherscan_link_tx_hash'),
  etherscanLinkAddress('etherscan_link_address'),
  etherscanLinkBlock('etherscan_link_block'),
  etherscanLinkContracts('etherscan_link_contracts'),
  blockscoutAssetIcons('blockscout_asset_icons'),
  blockscoutLinkTxHash('blockscout_link_tx_hash'),
  blockscoutLinkAddress('blockscout_link_address'),
  blockscoutLinkBlock('blockscout_link_block'),
  blockscoutLinkContracts('blockscout_link_contracts');

  const VendorFlag(this.wireValue);
  final String wireValue;
}

extension VendorFlagCodec on VendorFlag {
  static VendorFlag fromJson(Object? value) =>
      VendorFlag.values.singleWhere((entry) => entry.wireValue == value);
}

final class VendorFlagInfo {
  const VendorFlagInfo({
    required this.flag,
    required this.comment,
    required this.unfinished,
  });
  final VendorFlag flag;
  final String comment;
  final bool unfinished;
  factory VendorFlagInfo.fromJson(Map<String, Object?> json) => VendorFlagInfo(
    flag: VendorFlagCodec.fromJson(json['flag']),
    comment: json['comment'] as String,
    unfinished: json['unfinished'] as bool,
  );
  Map<String, Object?> toJson() => {
    'flag': flag.wireValue,
    'comment': comment,
    'unfinished': unfinished,
  };
}

final class VendorParams {
  const VendorParams({required this.flag});
  final VendorFlag flag;
  factory VendorParams.fromJson(Map<String, Object?> json) =>
      VendorParams(flag: VendorFlagCodec.fromJson(json['flag']));
  Map<String, Object?> toJson() => {'flag': flag.wireValue};
}

final class ViewWallet {
  const ViewWallet({required this.evmAddress});
  final ApiAddress evmAddress;
  factory ViewWallet.fromJson(Map<String, Object?> json) =>
      ViewWallet(evmAddress: json['evm_address'] as String);
  Map<String, Object?> toJson() => {'evm_address': evmAddress};
}

typedef WalletType = Map<String, Object?>;

Account decodeAccount(Object? value) =>
    Account.fromJson(value as Map<String, Object?>);
AccountAssetBalanceParams decodeAccountAssetBalanceParams(Object? value) =>
    AccountAssetBalanceParams.fromJson(value as Map<String, Object?>);
AccountAssetParams decodeAccountAssetParams(Object? value) =>
    AccountAssetParams.fromJson(value as Map<String, Object?>);
AccountBalance decodeAccountBalance(Object? value) =>
    AccountBalance.fromJson(value as Map<String, Object?>);
AccountBalances decodeAccountBalances(Object? value) =>
    AccountBalances.fromJson(value as Map<String, Object?>);
AccountBalancesParams decodeAccountBalancesParams(Object? value) =>
    AccountBalancesParams.fromJson(value as Map<String, Object?>);
AccountCreateParams decodeAccountCreateParams(Object? value) =>
    AccountCreateParams.fromJson(value as Map<String, Object?>);
AccountGroup decodeAccountGroup(Object? value) =>
    AccountGroup.fromJson(value as Map<String, Object?>);
AccountGroupCreate decodeAccountGroupCreate(Object? value) =>
    AccountGroupCreate.fromJson(value as Map<String, Object?>);
AccountGroupUpdate decodeAccountGroupUpdate(Object? value) =>
    AccountGroupUpdate.fromJson(value as Map<String, Object?>);
AccountIdentity decodeAccountIdentity(Object? value) => value as num;
AccountLayout decodeAccountLayout(Object? value) =>
    AccountLayout.fromJson(value as Map<String, Object?>);
AccountLayoutAccountEntry decodeAccountLayoutAccountEntry(Object? value) =>
    AccountLayoutAccountEntry.fromJson(value as Map<String, Object?>);
AccountLayoutGroupEntry decodeAccountLayoutGroupEntry(Object? value) =>
    AccountLayoutGroupEntry.fromJson(value as Map<String, Object?>);
AccountLayoutUpdate decodeAccountLayoutUpdate(Object? value) =>
    AccountLayoutUpdate.fromJson(value as Map<String, Object?>);
AccountParams decodeAccountParams(Object? value) =>
    AccountParams.fromJson(value as Map<String, Object?>);
AccountUpdate decodeAccountUpdate(Object? value) =>
    AccountUpdate.fromJson(value as Map<String, Object?>);
AccountUpdateParams decodeAccountUpdateParams(Object? value) =>
    AccountUpdateParams.fromJson(value as Map<String, Object?>);
ApiAddress decodeApiAddress(Object? value) => value as String;
ApiBytes decodeApiBytes(Object? value) => value as String;
ApiU256 decodeApiU256(Object? value) => value as String;
Asset decodeAsset(Object? value) =>
    Asset.fromJson(value as Map<String, Object?>);
AssetCreateParams decodeAssetCreateParams(Object? value) =>
    AssetCreateParams.fromJson(value as Map<String, Object?>);
AssetIdentity decodeAssetIdentity(Object? value) => value as String;
AssetMetadataDiscovery decodeAssetMetadataDiscovery(Object? value) =>
    AssetMetadataDiscovery.fromJson(value as Map<String, Object?>);
AssetMetadataOption decodeAssetMetadataOption(Object? value) =>
    AssetMetadataOption.fromJson(value as Map<String, Object?>);
AssetParams decodeAssetParams(Object? value) =>
    AssetParams.fromJson(value as Map<String, Object?>);
AssetQuoteParams decodeAssetQuoteParams(Object? value) =>
    AssetQuoteParams.fromJson(value as Map<String, Object?>);
AssetUpdate decodeAssetUpdate(Object? value) =>
    AssetUpdate.fromJson(value as Map<String, Object?>);
AssetUpdateParams decodeAssetUpdateParams(Object? value) =>
    AssetUpdateParams.fromJson(value as Map<String, Object?>);
DecodeParams decodeDecodeParams(Object? value) =>
    DecodeParams.fromJson(value as Map<String, Object?>);
DecodeTransactionRequest decodeDecodeTransactionRequest(Object? value) =>
    DecodeTransactionRequest.fromJson(value as Map<String, Object?>);
DecodeTransactionResponse decodeDecodeTransactionResponse(Object? value) =>
    DecodeTransactionResponse.fromJson(value as Map<String, Object?>);
Decoded decodeDecoded(Object? value) => value as Map<String, Object?>;
DecodedCall decodeDecodedCall(Object? value) =>
    DecodedCall.fromJson(value as Map<String, Object?>);
DecodedContract decodeDecodedContract(Object? value) =>
    DecodedContract.fromJson(value as Map<String, Object?>);
DecodedFunction decodeDecodedFunction(Object? value) =>
    DecodedFunction.fromJson(value as Map<String, Object?>);
DecodedParam decodeDecodedParam(Object? value) =>
    DecodedParam.fromJson(value as Map<String, Object?>);
DecodedProxy decodeDecodedProxy(Object? value) =>
    DecodedProxy.fromJson(value as Map<String, Object?>);
DeriveMnemonicInput decodeDeriveMnemonicInput(Object? value) =>
    DeriveMnemonicInput.fromJson(value as Map<String, Object?>);
DeriveMnemonicParams decodeDeriveMnemonicParams(Object? value) =>
    DeriveMnemonicParams.fromJson(value as Map<String, Object?>);
DeriveMnemonicResult decodeDeriveMnemonicResult(Object? value) =>
    DeriveMnemonicResult.fromJson(value as Map<String, Object?>);
DerivePrivateKeyParams decodeDerivePrivateKeyParams(Object? value) =>
    DerivePrivateKeyParams.fromJson(value as Map<String, Object?>);
EOAWallet decodeEOAWallet(Object? value) =>
    EOAWallet.fromJson(value as Map<String, Object?>);
EmptyParams decodeEmptyParams(Object? value) => const <String, Never>{};
EndpointCreateParams decodeEndpointCreateParams(Object? value) =>
    EndpointCreateParams.fromJson(value as Map<String, Object?>);
EndpointParams decodeEndpointParams(Object? value) =>
    EndpointParams.fromJson(value as Map<String, Object?>);
EndpointUpdateParams decodeEndpointUpdateParams(Object? value) =>
    EndpointUpdateParams.fromJson(value as Map<String, Object?>);
Erc4626QuoterConfig decodeErc4626QuoterConfig(Object? value) =>
    const <String, Never>{};
FixedQuoterConfig decodeFixedQuoterConfig(Object? value) =>
    FixedQuoterConfig.fromJson(value as Map<String, Object?>);
GroupCreateParams decodeGroupCreateParams(Object? value) =>
    GroupCreateParams.fromJson(value as Map<String, Object?>);
GroupIdentity decodeGroupIdentity(Object? value) => value as num;
GroupParams decodeGroupParams(Object? value) =>
    GroupParams.fromJson(value as Map<String, Object?>);
GroupUpdateParams decodeGroupUpdateParams(Object? value) =>
    GroupUpdateParams.fromJson(value as Map<String, Object?>);
JsonRpcVersion decodeJsonRpcVersion(Object? value) =>
    JsonRpcVersionCodec.fromJson(value);
LayoutUpdateParams decodeLayoutUpdateParams(Object? value) =>
    LayoutUpdateParams.fromJson(value as Map<String, Object?>);
Network decodeNetwork(Object? value) =>
    Network.fromJson(value as Map<String, Object?>);
NetworkCreateParams decodeNetworkCreateParams(Object? value) =>
    NetworkCreateParams.fromJson(value as Map<String, Object?>);
NetworkEndpoint decodeNetworkEndpoint(Object? value) =>
    NetworkEndpoint.fromJson(value as Map<String, Object?>);
NetworkEndpointUpdate decodeNetworkEndpointUpdate(Object? value) =>
    NetworkEndpointUpdate.fromJson(value as Map<String, Object?>);
NetworkIdentity decodeNetworkIdentity(Object? value) => value as num;
NetworkMetadataDiscovery decodeNetworkMetadataDiscovery(Object? value) =>
    NetworkMetadataDiscovery.fromJson(value as Map<String, Object?>);
NetworkMetadataOption decodeNetworkMetadataOption(Object? value) =>
    NetworkMetadataOption.fromJson(value as Map<String, Object?>);
NetworkParams decodeNetworkParams(Object? value) =>
    NetworkParams.fromJson(value as Map<String, Object?>);
NetworkUpdate decodeNetworkUpdate(Object? value) =>
    NetworkUpdate.fromJson(value as Map<String, Object?>);
NetworkUpdateParams decodeNetworkUpdateParams(Object? value) =>
    NetworkUpdateParams.fromJson(value as Map<String, Object?>);
Quoter decodeQuoter(Object? value) =>
    Quoter.fromJson(value as Map<String, Object?>);
QuoterConfig decodeQuoterConfig(Object? value) => value as Map<String, Object?>;
QuoterCreate decodeQuoterCreate(Object? value) =>
    QuoterCreate.fromJson(value as Map<String, Object?>);
QuoterCreateParams decodeQuoterCreateParams(Object? value) =>
    QuoterCreateParams.fromJson(value as Map<String, Object?>);
QuoterDiscoverParams decodeQuoterDiscoverParams(Object? value) =>
    QuoterDiscoverParams.fromJson(value as Map<String, Object?>);
QuoterDiscovery decodeQuoterDiscovery(Object? value) =>
    QuoterDiscovery.fromJson(value as Map<String, Object?>);
QuoterDiscoveryResponse decodeQuoterDiscoveryResponse(Object? value) =>
    QuoterDiscoveryResponse.fromJson(value as Map<String, Object?>);
QuoterParams decodeQuoterParams(Object? value) =>
    QuoterParams.fromJson(value as Map<String, Object?>);
QuoterUpdate decodeQuoterUpdate(Object? value) =>
    QuoterUpdate.fromJson(value as Map<String, Object?>);
QuoterUpdateParams decodeQuoterUpdateParams(Object? value) =>
    QuoterUpdateParams.fromJson(value as Map<String, Object?>);
RailgunWallet decodeRailgunWallet(Object? value) =>
    RailgunWallet.fromJson(value as Map<String, Object?>);
RawCall decodeRawCall(Object? value) =>
    RawCall.fromJson(value as Map<String, Object?>);
RpcCallSample decodeRpcCallSample(Object? value) =>
    RpcCallSample.fromJson(value as Map<String, Object?>);
RpcEndpointStats decodeRpcEndpointStats(Object? value) =>
    RpcEndpointStats.fromJson(value as Map<String, Object?>);
RpcErrorData decodeRpcErrorData(Object? value) =>
    RpcErrorData.fromJson(value as Map<String, Object?>);
RpcErrorEnvelope decodeRpcErrorEnvelope(Object? value) =>
    RpcErrorEnvelope.fromJson(value as Map<String, Object?>);
RpcErrorKind decodeRpcErrorKind(Object? value) =>
    RpcErrorKindCodec.fromJson(value);
RpcErrorObject decodeRpcErrorObject(Object? value) =>
    RpcErrorObject.fromJson(value as Map<String, Object?>);
RpcIdentity decodeRpcIdentity(Object? value) => value as Map<String, Object?>;
RpcMethodStats decodeRpcMethodStats(Object? value) =>
    RpcMethodStats.fromJson(value as Map<String, Object?>);
RpcPoolEndpointStats decodeRpcPoolEndpointStats(Object? value) =>
    RpcPoolEndpointStats.fromJson(value as Map<String, Object?>);
RpcPoolStats decodeRpcPoolStats(Object? value) =>
    RpcPoolStats.fromJson(value as Map<String, Object?>);
RpcRequestEnvelope decodeRpcRequestEnvelope(Object? value) =>
    RpcRequestEnvelope.fromJson(value as Map<String, Object?>);
RpcResponseEnvelope decodeRpcResponseEnvelope(Object? value) =>
    value as Map<String, Object?>;
RpcStatus decodeRpcStatus(Object? value) => value as Map<String, Object?>;
RpcStatusAlive decodeRpcStatusAlive(Object? value) =>
    RpcStatusAlive.fromJson(value as Map<String, Object?>);
RpcStatusDead decodeRpcStatusDead(Object? value) =>
    RpcStatusDead.fromJson(value as Map<String, Object?>);
RpcStatusDisabled decodeRpcStatusDisabled(Object? value) =>
    RpcStatusDisabled.fromJson(value as Map<String, Object?>);
RpcSuccessEnvelope decodeRpcSuccessEnvelope(Object? value) =>
    RpcSuccessEnvelope.fromJson(value as Map<String, Object?>);
SafeWallet decodeSafeWallet(Object? value) =>
    SafeWallet.fromJson(value as Map<String, Object?>);
SafeWalletTxExtra decodeSafeWalletTxExtra(Object? value) =>
    SafeWalletTxExtra.fromJson(value as Map<String, Object?>);
SignatureFallback decodeSignatureFallback(Object? value) =>
    SignatureFallback.fromJson(value as Map<String, Object?>);
SimulateParams decodeSimulateParams(Object? value) =>
    SimulateParams.fromJson(value as Map<String, Object?>);
SimulateTransactionRequest decodeSimulateTransactionRequest(Object? value) =>
    SimulateTransactionRequest.fromJson(value as Map<String, Object?>);
SimulateTransactionResponse decodeSimulateTransactionResponse(Object? value) =>
    SimulateTransactionResponse.fromJson(value as Map<String, Object?>);
Tx decodeTx(Object? value) => Tx.fromJson(value as Map<String, Object?>);
TxExtra decodeTxExtra(Object? value) =>
    TxExtra.fromJson(value as Map<String, Object?>);
UniswapV2Pair decodeUniswapV2Pair(Object? value) =>
    UniswapV2Pair.fromJson(value as Map<String, Object?>);
UniswapV2QuoterConfig decodeUniswapV2QuoterConfig(Object? value) =>
    UniswapV2QuoterConfig.fromJson(value as Map<String, Object?>);
UniswapV3Pool decodeUniswapV3Pool(Object? value) =>
    UniswapV3Pool.fromJson(value as Map<String, Object?>);
UniswapV3QuoterConfig decodeUniswapV3QuoterConfig(Object? value) =>
    UniswapV3QuoterConfig.fromJson(value as Map<String, Object?>);
VendorFlag decodeVendorFlag(Object? value) => VendorFlagCodec.fromJson(value);
VendorFlagInfo decodeVendorFlagInfo(Object? value) =>
    VendorFlagInfo.fromJson(value as Map<String, Object?>);
VendorParams decodeVendorParams(Object? value) =>
    VendorParams.fromJson(value as Map<String, Object?>);
ViewWallet decodeViewWallet(Object? value) =>
    ViewWallet.fromJson(value as Map<String, Object?>);
WalletType decodeWalletType(Object? value) => value as Map<String, Object?>;
