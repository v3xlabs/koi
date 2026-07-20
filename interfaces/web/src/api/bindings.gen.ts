/* Generated from the Rust RPC contract. Do not edit. */

export type Account = { account_identity: AccountIdentity, name: string, networks: Array<NetworkIdentity>, metadata: WalletType, group_id?: GroupIdentity, display_order: number, };

export type AccountAssetBalanceParams = { account_identity: AccountIdentity, asset_identity: AssetIdentity, display_currency: AssetIdentity, };

export type AccountAssetParams = { account_identity: AccountIdentity, asset_identity: AssetIdentity, };

export type AccountBalance = { asset_identity: AssetIdentity, balance?: string, balance_error?: string, asset_quote?: string, asset_quote_error?: string, asset_24h_quote?: string, asset_24h_quote_error?: string, balance_quote?: string, balance_quote_error?: string, updated_at: string, };

export type AccountBalances = { balances: Array<AccountBalance>, total_quote?: string, updated_at: string, asset: AssetIdentity, errors: Array<string>, };

export type AccountBalancesParams = { account_identity: AccountIdentity, display_currency: AssetIdentity, fresh?: boolean, };

export type AccountCreateParams = { input: Account, };

export type AccountGroup = { group_identity: GroupIdentity, name: string, display_order: number, };

export type AccountGroupCreate = { name: string, };

export type AccountGroupUpdate = { name?: string, };

export type AccountIdentity = number;

export type AccountLayout = { groups: Array<AccountGroup>, accounts: Array<Account>, };

export type AccountLayoutAccountEntry = { account_identity: AccountIdentity, group_id?: GroupIdentity, display_order: number, };

export type AccountLayoutGroupEntry = { group_identity: GroupIdentity, name: string, display_order: number, };

export type AccountLayoutUpdate = { groups: Array<AccountLayoutGroupEntry>, accounts: Array<AccountLayoutAccountEntry>, };

export type AccountParams = { account_identity: AccountIdentity, };

export type AccountUpdate = { name?: string, networks?: Array<NetworkIdentity>, metadata?: WalletType, };

export type AccountUpdateParams = { account_identity: AccountIdentity, input: AccountUpdate, };

export type ApiAddress = string;

export type ApiBytes = string;

export type ApiU256 = string;

export type Asset = { asset_identity: AssetIdentity, asset_name: string, asset_symbol: string, asset_decimals: number, asset_icon_url?: string, };

export type AssetCreateParams = { input: Asset, };

export type AssetIdentity = string;

export type AssetMetadataDiscovery = { asset_identity: AssetIdentity, options: Record<string, AssetMetadataOption>, };

export type AssetMetadataOption = { name?: string, symbol?: string, decimals?: number, icon_url?: string, };

export type AssetParams = { asset_identity: AssetIdentity, };

export type AssetQuoteParams = { asset_identity: AssetIdentity, display_asset?: AssetIdentity, };

export type AssetUpdate = { asset_name?: string, asset_symbol?: string, asset_decimals?: number, asset_icon_url?: string, };

export type AssetUpdateParams = { asset_identity: AssetIdentity, input: AssetUpdate, };

export type DecodeParams = { network_identity: NetworkIdentity, input: DecodeTransactionRequest, };

export type DecodeTransactionRequest = { from?: ApiAddress, to: ApiAddress, value?: ApiU256, data?: ApiBytes, };

export type DecodeTransactionResponse = { call: DecodedCall, };

export type Decoded = { "kind": "verified" } & DecodedFunction | { "kind": "signature_fallback" } & SignatureFallback | { "kind": "raw" } & RawCall;

export type DecodedCall = { from?: ApiAddress, to: ApiAddress, value: ApiU256, operation?: string, data: ApiBytes, selector?: ApiBytes, decoded: Decoded, subcalls?: Array<DecodedCall>, };

export type DecodedContract = { address: ApiAddress, verified_name?: string, proxy?: DecodedProxy, };

export type DecodedFunction = { contract: DecodedContract, selector: ApiBytes, function: string, signature: string, params: Array<DecodedParam>, };

export type DecodedParam = { name?: string, ty: string, value: unknown, };

export type DecodedProxy = { proxy_type?: string, implementation: ApiAddress, implementation_name?: string, };

export type DeriveMnemonicInput = { mnemonic: string, paths: Array<string>, };

export type DeriveMnemonicParams = { input: DeriveMnemonicInput, };

export type DeriveMnemonicResult = { path: string, address: string, };

export type DerivePrivateKeyParams = { input: string, };

export type EOAWallet = { evm_address: ApiAddress, };

export type EmptyParams = Record<string, never>;

export type EndpointCreateParams = { network_identity: NetworkIdentity, input: NetworkEndpoint, };

export type EndpointParams = { network_identity: NetworkIdentity, endpoint_identity: number, };

export type EndpointUpdateParams = { network_identity: NetworkIdentity, endpoint_identity: number, input: NetworkEndpointUpdate, };

export type Erc4626QuoterConfig = Record<string, never>;

export type FixedQuoterConfig = { price: string, decimals: number, token_in_decimals: number, token_out_decimals: number, };

export type GroupCreateParams = { input: AccountGroupCreate, };

export type GroupIdentity = number;

export type GroupParams = { group_identity: GroupIdentity, };

export type GroupUpdateParams = { group_identity: GroupIdentity, input: AccountGroupUpdate, };

export type JsonRpcVersion = "2.0";

export type LayoutUpdateParams = { input: AccountLayoutUpdate, };

export type Network = {
/**
 * evm chain id
 */
network_identity: NetworkIdentity,
/**
 * name, Ethereum Mainnet, Optimism, etc
 */
network_name: string,
/**
 * icon url, https://example.com/icon.png, etc
 */
network_icon_url?: string, };

export type NetworkCreateParams = { input: Network, };

export type NetworkEndpoint = { endpoint_identity: number, endpoint_label?: string, endpoint_type: string, endpoint_url: string, endpoint_disabled: boolean, network_identity: NetworkIdentity, };

export type NetworkEndpointUpdate = { endpoint_label?: string, endpoint_type?: string, endpoint_url?: string, endpoint_disabled?: boolean, };

export type NetworkIdentity = number;

export type NetworkMetadataDiscovery = { network_identity: NetworkIdentity, options: Record<string, NetworkMetadataOption>, };

export type NetworkMetadataOption = { icon_url?: string, };

export type NetworkParams = { network_identity: NetworkIdentity, };

export type NetworkUpdate = { network_name?: string, network_icon_url?: string, };

export type NetworkUpdateParams = { network_identity: NetworkIdentity, input: NetworkUpdate, };

export type Quoter = { quoter_identity: string, quoter_name: string, token_a: AssetIdentity, token_b: AssetIdentity, config: QuoterConfig, enabled: boolean, watch: boolean, };

export type QuoterConfig = { "type": "fixed" } & FixedQuoterConfig | { "type": "erc4626" } & Erc4626QuoterConfig | { "type": "uniswap_v2" } & UniswapV2QuoterConfig | { "type": "uniswap_v3" } & UniswapV3QuoterConfig;

export type QuoterCreate = { quoter_name: string, token_a: AssetIdentity, token_b: AssetIdentity, config: QuoterConfig, enabled: boolean, watch: boolean, };

export type QuoterCreateParams = { input: QuoterCreate, };

export type QuoterDiscoverParams = { input: QuoterDiscovery, };

export type QuoterDiscovery = { token_a: AssetIdentity, token_b?: AssetIdentity, };

export type QuoterDiscoveryResponse = { erc4626?: AssetIdentity, uniswap_v2?: UniswapV2Pair, uniswap_v3?: Array<UniswapV3Pool>, };

export type QuoterParams = { quoter_identity: string, };

export type QuoterUpdate = { quoter_name?: string, token_a?: AssetIdentity, token_b?: AssetIdentity, config?: QuoterConfig, enabled?: boolean, watch?: boolean, };

export type QuoterUpdateParams = { quoter_identity: string, input: QuoterUpdate, };

export type RailgunWallet = { railgun_address: string, };

export type RawCall = { data: ApiBytes, };

export type RpcCallSample = { timestamp: number, methods: Array<string>, request_count: number, duration_ms: number, success: boolean, error?: string, };

export type RpcEndpointStats = { in_flight: number, queued: number, max_in_flight: number, total_requests: number, total_errors: number, total_rate_limited: number, average_duration_ms: number, connection_attempts: number, connection_successes: number, connection_failures: number, last_request_at?: number, last_error_at?: number, last_connected_at?: number, last_error?: string, methods: Array<RpcMethodStats>, recent: Array<RpcCallSample>, };

export type RpcErrorData = { kind: RpcErrorKind, message: string, };

export type RpcErrorEnvelope = { jsonrpc: JsonRpcVersion, id: RpcIdentity, error: RpcErrorObject, };

export type RpcErrorKind = "invalid_input" | "not_found" | "conflict" | "unavailable" | "internal";

export type RpcErrorObject = { code: number, message: string, data?: RpcErrorData, };

export type RpcIdentity = number | string | null;

export type RpcMethodStats = { method: string, total: number, errors: number, };

export type RpcPoolEndpointStats = { endpoint_identity: number, status: string, in_flight: number, queued: number, total_requests: number, total_errors: number, };

export type RpcPoolStats = { network_identity: NetworkIdentity, endpoint_count: number, alive_count: number, dead_count: number, disabled_count: number, in_flight: number, queued: number, total_requests: number, total_errors: number, endpoints: Array<RpcPoolEndpointStats>, };

export type RpcRequestEnvelope = { jsonrpc: JsonRpcVersion, id?: RpcIdentity, method: string, params: Record<string, unknown>, };

export type RpcResponseEnvelope = RpcSuccessEnvelope | RpcErrorEnvelope;

export type RpcStatus = { "status": "Alive" } & RpcStatusAlive | { "status": "Dead" } & RpcStatusDead | { "status": "Disabled" } & RpcStatusDisabled;

export type RpcStatusAlive = { block_number: number, network_identity: NetworkIdentity, timestamp: number, rpc: RpcEndpointStats, };

export type RpcStatusDead = { error: string, rpc: RpcEndpointStats, };

export type RpcStatusDisabled = { rpc: RpcEndpointStats, };

export type RpcSuccessEnvelope = { jsonrpc: JsonRpcVersion, id: RpcIdentity, result: unknown, };

export type SafeWallet = { evm_address: ApiAddress, };

export type SafeWalletTxExtra = { nonce?: number, execution_date?: string, safe_tx_hash?: ApiBytes, proposer?: ApiAddress, executor?: ApiAddress, is_successful?: boolean, is_executed?: boolean, origin?: string, extra: unknown, };

export type SignatureFallback = { contract?: DecodedContract, selector: ApiBytes, candidates: Array<string>, };

export type SimulateParams = { network_identity: NetworkIdentity, input: SimulateTransactionRequest, };

export type SimulateTransactionRequest = { from?: ApiAddress, to: ApiAddress, value?: ApiU256, data?: ApiBytes, };

export type SimulateTransactionResponse = { call: DecodedCall, };

export type Tx = { network_identity: NetworkIdentity, tx_hash?: ApiBytes, from?: ApiAddress, to?: ApiAddress, data?: ApiBytes, value: ApiU256, decoded?: DecodedCall, extra: TxExtra, };

export type TxExtra = { safe_wallet?: SafeWalletTxExtra, };

export type UniswapV2Pair = { pair_address: string, reserve_0?: string, reserve_1?: string, };

export type UniswapV2QuoterConfig = { pair_address: string, };

export type UniswapV3Pool = { pool_address: string, fee: number, reserve_0?: string, reserve_1?: string, };

export type UniswapV3QuoterConfig = { pool_address: string, };

export type VendorFlag = "avara_asset_icons" | "zerion_asset_icons" | "smoldapp_asset_icons" | "smoldapp_network_icons" | "safewallet_network_icons" | "safewallet_asset_icons" | "safewallet_transactions_api" | "etherscan_asset_icons" | "etherscan_link_tx_hash" | "etherscan_link_address" | "etherscan_link_block" | "etherscan_link_contracts" | "blockscout_asset_icons" | "blockscout_link_tx_hash" | "blockscout_link_address" | "blockscout_link_block" | "blockscout_link_contracts";

export type VendorFlagInfo = { flag: VendorFlag, comment: string, unfinished: boolean, };

export type VendorParams = { flag: VendorFlag, };

export type ViewWallet = { evm_address: ApiAddress, };

export type WalletType = { "type": "safe" } & SafeWallet | { "type": "eoa" } & EOAWallet | { "type": "view" } & ViewWallet | { "type": "railgun" } & RailgunWallet;

export type AccountAssetAddRpcParams = AccountAssetParams;
export type AccountAssetAddRpcResult = null;
export type AccountAssetBalanceRpcParams = AccountAssetBalanceParams;
export type AccountAssetBalanceRpcResult = AccountBalance;
export type AccountAssetListRpcParams = AccountParams;
export type AccountAssetListRpcResult = Array<AssetIdentity>;
export type AccountAssetRemoveRpcParams = AccountAssetParams;
export type AccountAssetRemoveRpcResult = null;
export type AccountBalanceListRpcParams = AccountBalancesParams;
export type AccountBalanceListRpcResult = AccountBalances;
export type AccountCreateRpcParams = AccountCreateParams;
export type AccountCreateRpcResult = Account;
export type AccountDeleteRpcParams = AccountParams;
export type AccountDeleteRpcResult = null;
export type AccountDerivationDefaultPathRpcParams = EmptyParams;
export type AccountDerivationDefaultPathRpcResult = string;
export type AccountDerivationFromMnemonicRpcParams = DeriveMnemonicParams;
export type AccountDerivationFromMnemonicRpcResult = Array<DeriveMnemonicResult>;
export type AccountDerivationFromPrivateKeyRpcParams = DerivePrivateKeyParams;
export type AccountDerivationFromPrivateKeyRpcResult = string;
export type AccountGetRpcParams = AccountParams;
export type AccountGetRpcResult = Account;
export type AccountGroupCreateRpcParams = GroupCreateParams;
export type AccountGroupCreateRpcResult = AccountGroup;
export type AccountGroupDeleteRpcParams = GroupParams;
export type AccountGroupDeleteRpcResult = null;
export type AccountGroupUpdateRpcParams = GroupUpdateParams;
export type AccountGroupUpdateRpcResult = AccountGroup;
export type AccountLayoutGetRpcParams = EmptyParams;
export type AccountLayoutGetRpcResult = AccountLayout;
export type AccountLayoutUpdateRpcParams = LayoutUpdateParams;
export type AccountLayoutUpdateRpcResult = AccountLayout;
export type AccountListRpcParams = EmptyParams;
export type AccountListRpcResult = Array<Account>;
export type AccountMnemonicGenerateRpcParams = EmptyParams;
export type AccountMnemonicGenerateRpcResult = string;
export type AccountNextIdentityRpcParams = EmptyParams;
export type AccountNextIdentityRpcResult = AccountIdentity;
export type AccountTransactionListRpcParams = AccountParams;
export type AccountTransactionListRpcResult = Array<Tx>;
export type AccountTransactionPendingRpcParams = AccountParams;
export type AccountTransactionPendingRpcResult = Array<Tx>;
export type AccountUpdateRpcParams = AccountUpdateParams;
export type AccountUpdateRpcResult = Account;
export type AssetCreateRpcParams = AssetCreateParams;
export type AssetCreateRpcResult = Asset;
export type AssetDeleteRpcParams = AssetParams;
export type AssetDeleteRpcResult = null;
export type AssetDiscoverMetadataRpcParams = AssetParams;
export type AssetDiscoverMetadataRpcResult = AssetMetadataDiscovery;
export type AssetGetRpcParams = AssetParams;
export type AssetGetRpcResult = Asset;
export type AssetListRpcParams = EmptyParams;
export type AssetListRpcResult = Array<Asset>;
export type AssetQuoteRpcParams = AssetQuoteParams;
export type AssetQuoteRpcResult = string;
export type AssetUpdateRpcParams = AssetUpdateParams;
export type AssetUpdateRpcResult = Asset;
export type NetworkCreateRpcParams = NetworkCreateParams;
export type NetworkCreateRpcResult = Network;
export type NetworkDeleteRpcParams = NetworkParams;
export type NetworkDeleteRpcResult = null;
export type NetworkDiscoverMetadataRpcParams = NetworkParams;
export type NetworkDiscoverMetadataRpcResult = NetworkMetadataDiscovery;
export type EndpointCreateRpcParams = EndpointCreateParams;
export type EndpointCreateRpcResult = NetworkEndpoint;
export type EndpointDeleteRpcParams = EndpointParams;
export type EndpointDeleteRpcResult = null;
export type EndpointGetRpcParams = EndpointParams;
export type EndpointGetRpcResult = NetworkEndpoint;
export type EndpointListRpcParams = NetworkParams;
export type EndpointListRpcResult = Array<NetworkEndpoint>;
export type EndpointNextIdentityRpcParams = NetworkParams;
export type EndpointNextIdentityRpcResult = number;
export type EndpointStatusRpcParams = EndpointParams;
export type EndpointStatusRpcResult = RpcStatus;
export type EndpointUpdateRpcParams = EndpointUpdateParams;
export type EndpointUpdateRpcResult = NetworkEndpoint;
export type NetworkGetRpcParams = NetworkParams;
export type NetworkGetRpcResult = Network;
export type NetworkListRpcParams = EmptyParams;
export type NetworkListRpcResult = Array<Network>;
export type NetworkListPresetsRpcParams = EmptyParams;
export type NetworkListPresetsRpcResult = Array<Network>;
export type NetworkRpcStatsRpcParams = NetworkParams;
export type NetworkRpcStatsRpcResult = RpcPoolStats;
export type NetworkUpdateRpcParams = NetworkUpdateParams;
export type NetworkUpdateRpcResult = Network;
export type QuoterCreateRpcParams = QuoterCreateParams;
export type QuoterCreateRpcResult = Quoter;
export type QuoterDiscoverRpcParams = QuoterDiscoverParams;
export type QuoterDiscoverRpcResult = QuoterDiscoveryResponse;
export type QuoterGetRpcParams = QuoterParams;
export type QuoterGetRpcResult = Quoter;
export type QuoterListRpcParams = EmptyParams;
export type QuoterListRpcResult = Array<Quoter>;
export type QuoterUpdateRpcParams = QuoterUpdateParams;
export type QuoterUpdateRpcResult = Quoter;
export type SystemPingRpcParams = EmptyParams;
export type SystemPingRpcResult = string;
export type TransactionDecodeRpcParams = DecodeParams;
export type TransactionDecodeRpcResult = DecodeTransactionResponse;
export type TransactionSimulateRpcParams = SimulateParams;
export type TransactionSimulateRpcResult = SimulateTransactionResponse;
export type VendorDisableRpcParams = VendorParams;
export type VendorDisableRpcResult = null;
export type VendorEnableRpcParams = VendorParams;
export type VendorEnableRpcResult = null;
export type VendorListAllRpcParams = EmptyParams;
export type VendorListAllRpcResult = Array<VendorFlagInfo>;
export type VendorListEnabledRpcParams = EmptyParams;
export type VendorListEnabledRpcResult = Array<VendorFlag>;
