/* This file is generated from the Rust RPC contract. */

export type RpcErrorKind = "invalid_input" | "not_found" | "conflict" | "unavailable" | "internal";
export type RpcErrorData = { kind: RpcErrorKind; message: string };
export type RpcErrorObject = { code: number; message: string; data?: RpcErrorData | null };

export type WalletType =
    | { type: "safe"; evm_address: string }
    | { type: "eoa"; evm_address: string }
    | { type: "view"; evm_address: string }
    | { type: "railgun"; railgun_address: string };

export type Account = {
    account_identity: number;
    name: string;
    networks: number[];
    metadata: WalletType;
    group_id?: number;
    display_order: number;
};
export type AccountUpdate = { name?: string; networks?: number[]; metadata?: WalletType };
export type AccountGroup = { group_identity: number; name: string; display_order: number };
export type AccountGroupCreate = { name: string };
export type AccountGroupUpdate = { name?: string };
export type AccountLayout = { groups: AccountGroup[]; accounts: Account[] };
export type AccountLayoutUpdate = {
    groups: { group_identity: number; name: string; display_order: number }[];
    accounts: { account_identity: number; group_id?: number; display_order: number }[];
};
export type AccountBalance = {
    asset_identity: string;
    balance?: string;
    balance_error?: string;
    asset_quote?: string;
    asset_quote_error?: string;
    asset_24h_quote?: string;
    asset_24h_quote_error?: string;
    balance_quote?: string;
    balance_quote_error?: string;
    updated_at: string;
};
export type AccountBalances = { balances: AccountBalance[]; total_quote?: string; updated_at: string; asset: string; errors: string[] };

export type Asset = { asset_identity: string; asset_name: string; asset_symbol: string; asset_decimals: number; asset_icon_url?: string };
export type AssetUpdate = { asset_name?: string; asset_symbol?: string; asset_decimals?: number; asset_icon_url?: string };
export type AssetMetadataOption = { name?: string; symbol?: string; decimals?: number; icon_url?: string };
export type AssetMetadataDiscovery = { asset_identity: string; options: Record<string, AssetMetadataOption> };

export type Network = { network_identity: number; network_name: string; network_icon_url?: string };
export type NetworkUpdate = { network_name?: string; network_icon_url?: string };
export type NetworkEndpoint = { endpoint_identity: number; endpoint_label?: string; endpoint_type: string; endpoint_url: string; endpoint_disabled: boolean; network_identity: number };
export type NetworkEndpointUpdate = { endpoint_label?: string; endpoint_type?: string; endpoint_url?: string; endpoint_disabled?: boolean };
export type NetworkMetadataDiscovery = { network_identity: number; options: Record<string, { icon_url?: string }> };
export type RpcMethodStats = { method: string; total: number; errors: number };
export type RpcCallSample = { timestamp: number; methods: string[]; request_count: number; duration_ms: number; success: boolean; error?: string | null };
export type RpcEndpointStats = {
    in_flight: number; queued: number; max_in_flight: number; total_requests: number; total_errors: number;
    total_rate_limited: number; average_duration_ms: number; connection_attempts: number;
    connection_successes: number; connection_failures: number; last_request_at?: number;
    last_error_at?: number; last_connected_at?: number; last_error?: string;
    methods: RpcMethodStats[]; recent: RpcCallSample[];
};
export type RpcPoolStats = {
    network_identity: number; endpoint_count: number; alive_count: number; dead_count: number;
    disabled_count: number; in_flight: number; queued: number; total_requests: number; total_errors: number;
    endpoints: { endpoint_identity: number; status: string; in_flight: number; queued: number; total_requests: number; total_errors: number }[];
};
export type RpcStatus =
    | ({ status: "Alive"; block_number: number; network_identity: number; timestamp: number; rpc: RpcEndpointStats })
    | ({ status: "Dead"; error: string; rpc: RpcEndpointStats })
    | ({ status: "Disabled"; rpc: RpcEndpointStats });

export type QuoterConfig =
    | { type: "fixed"; price: string; decimals: number; token_in_decimals: number; token_out_decimals: number }
    | { type: "erc4626" }
    | { type: "uniswap_v2"; pair_address: string }
    | { type: "uniswap_v3"; pool_address: string };
export type Quoter = { quoter_identity: string; quoter_name: string; token_a: string; token_b: string; config: QuoterConfig; enabled: boolean; watch: boolean };
export type QuoterCreate = Omit<Quoter, "quoter_identity">;
export type QuoterUpdate = Partial<QuoterCreate>;
export type UniswapV2Pair = { pair_address: string; reserve_0?: string | null; reserve_1?: string | null };
export type UniswapV3Pool = { pool_address: string; fee: number; reserve_0?: string | null; reserve_1?: string | null };
export type QuoterDiscovery = { token_a: string; token_b?: string };
export type QuoterDiscoveryResponse = { erc4626?: string; uniswap_v2?: UniswapV2Pair; uniswap_v3?: UniswapV3Pool[] };

export type VendorFlag = "avara_asset_icons" | "zerion_asset_icons" | "smoldapp_asset_icons" | "smoldapp_network_icons" | "safewallet_network_icons" | "safewallet_asset_icons" | "safewallet_transactions_api" | "etherscan_asset_icons" | "etherscan_link_tx_hash" | "etherscan_link_address" | "etherscan_link_block" | "etherscan_link_contracts" | "blockscout_asset_icons" | "blockscout_link_tx_hash" | "blockscout_link_address" | "blockscout_link_block" | "blockscout_link_contracts";
export type VendorFlagInfo = { flag: VendorFlag; comment: string; unfinished: boolean };

export type DecodedContract = { address: string; verified_name?: string; proxy?: { proxy_type?: string; implementation: string; implementation_name?: string } };
export type DecodedParam = { name?: string; ty: string; value: unknown };
export type Decoded =
    | { kind: "verified"; contract: DecodedContract; selector: string; function: string; signature: string; params: DecodedParam[] }
    | { kind: "signature_fallback"; contract?: DecodedContract; selector: string; candidates: string[] }
    | { kind: "raw"; data: string };
export type DecodedCall = { from?: string; to: string; value: string; operation?: string; data: string; selector?: string; decoded: Decoded; subcalls: DecodedCall[] };
export type Tx = {
    network_identity: number; tx_hash?: string; from?: string; to?: string;
    data?: string; value: string; decoded?: DecodedCall;
    extra: { safe_wallet?: { nonce?: number; execution_date?: string; safe_tx_hash?: string; proposer?: string; executor?: string; is_successful?: boolean; is_executed?: boolean; origin?: string; extra: unknown } };
};
export type DecodeTransactionRequest = { from?: string; to: string; value?: string; data?: string };
export type DecodeTransactionResponse = { call: DecodedCall };
export type SimulateTransactionRequest = DecodeTransactionRequest;
export type SimulateTransactionResponse = DecodeTransactionResponse;
export type DeriveMnemonicResult = { path: string; address: string };
