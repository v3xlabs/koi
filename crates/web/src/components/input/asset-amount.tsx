import { Dialog } from "@kobalte/core/dialog";
import { FiCheck, FiChevronDown, FiSearch } from "solid-icons/fi";
import { Accessor, Component, createMemo, createSignal, For, Show, useContext } from "solid-js";
import { formatUnits } from "viem";

import { appcontext } from "#/api";
import { useAccountAssetBalance, useAccountBalances } from "#/api/account";
import { Asset, useAsset, useAssets } from "#/api/asset";
import { AssetIcon } from "#/components/asset/icon";
import { formatAmount } from "#/utils/units";

type AssetGroup = {
    label: string;
    sort: number;
    options: Asset[];
};

const assetGroup = (asset: Asset): Omit<AssetGroup, "options"> => {
    const [kind, chainId] = asset.asset_identity.split(":");

    if (kind === "fiat") {
        return { label: "Fiat", sort: -1 };
    }

    if ((kind === "native" || kind === "erc20") && chainId) {
        return { label: `EVM Chain #${chainId}`, sort: Number(chainId) };
    }

    return { label: "Other", sort: Number.MAX_SAFE_INTEGER };
};

export type CombinedAssetAmountInputProps = {
    amount: Accessor<string>;
    onAmountChange: (value: string) => void;
    asset: Accessor<string>;
    onAssetChange: (value: string) => void;
    networkIdentity?: number;
    accountIdentity?: number;
    placeholder?: string;
    balanceLabel?: string;
};

export const CombinedAssetAmountInput: Component<CombinedAssetAmountInputProps> = (props) => {
    const [isOpen, setIsOpen] = createSignal(false);
    const [searchQuery, setSearchQuery] = createSignal("");
    const { displayCurrency: [displayCurrency] } = useContext(appcontext);

    const hasAsset = () => !!props.asset();
    const assetIdentity = () => props.asset() || "native:1";

    const assetQuery = useAsset(
        () => ({ path: { asset_identity: assetIdentity() } }),
        { enabled: hasAsset() },
    );
    const balanceQuery = useAccountAssetBalance(
        () => ({
            path: { account_identity: props.accountIdentity as number, asset_identity: assetIdentity() },
            query: { display_currency: displayCurrency() },
        }),
        { enabled: () => hasAsset() && props.accountIdentity !== undefined },
    );

    const balanceHuman = createMemo(() => {
        const b = balanceQuery.data?.balance;
        const d = assetQuery.data?.asset_decimals;
        const s = assetQuery.data?.asset_symbol;

        if (!b || d === undefined) return undefined;

        const bigint = BigInt(b);

        return {
            value: formatUnits(bigint, d),
            display: formatAmount(bigint, { decimals: d, precision: 2 }),
            symbol: s,
        };
    });

    const assetsQuery = useAssets();
    const balancesQuery = useAccountBalances(
        () => ({
            path: { account_identity: props.accountIdentity as number },
            query: { display_currency: displayCurrency() },
        }),
        { enabled: () => props.accountIdentity !== undefined },
    );

    const allAssets = createMemo(() => assetsQuery.data?.assets ?? []);

    const assets = createMemo(() => {
        let list = allAssets();

        if (props.networkIdentity !== undefined) {
            const networkId = props.networkIdentity.toString();

            list = list.filter((asset) => {
                const [kind, chainId] = asset.asset_identity.split(":");

                if (kind === "fiat") return false;

                return chainId === networkId;
            });
        }

        const balances = balancesQuery.data?.balances;

        if (balances) {
            const balanceMap = new Map(balances.map(b => [b.asset_identity, b]));

            list = list.filter(a => {
                const b = balanceMap.get(a.asset_identity);

                return b && b.balance && BigInt(b.balance) > 0n;
            });

            list = list.toSorted((a, b) => {
                const qa = balanceMap.get(a.asset_identity)?.balance_quote;
                const qb = balanceMap.get(b.asset_identity)?.balance_quote;

                if (qa && qb) {
                    const diff = BigInt(qb) - BigInt(qa);

                    return diff > 0n ? 1 : diff < 0n ? -1 : 0;
                }

                if (qa) return -1;

                if (qb) return 1;

                return a.asset_name.localeCompare(b.asset_name);
            });
        }

        return list;
    });

    const filteredAssets = createMemo(() => {
        const query = searchQuery().trim().toLowerCase();

        if (!query) return assets();

        return assets().filter(asset =>
            asset.asset_name.toLowerCase().includes(query)
            || asset.asset_symbol.toLowerCase().includes(query)
            || asset.asset_identity.toLowerCase().includes(query),
        );
    });

    const assetGroups = createMemo(() => {
        const hasBalances = !!balancesQuery.data?.balances;

        if (hasBalances) {
            return [{ label: "", sort: 0, options: filteredAssets() }];
        }

        const groups = new Map<string, AssetGroup>();

        for (const asset of filteredAssets()) {
            const group = assetGroup(asset);
            const existing = groups.get(group.label);

            if (existing) {
                existing.options.push(asset);
            }
            else {
                groups.set(group.label, { ...group, options: [asset] });
            }
        }

        return [...groups.values()]
            .map(group => ({
                ...group,
                options: group.options.toSorted((a, b) => a.asset_name.localeCompare(b.asset_name)),
            }))
            .toSorted((a, b) => a.sort - b.sort || a.label.localeCompare(b.label));
    });

    const balancesMap = createMemo(() => {
        const balances = balancesQuery.data?.balances;

        if (!balances) return undefined;

        return new Map(balances.map(b => [b.asset_identity, b]));
    });

    const selectedAsset = createMemo(() => {
        if (!props.asset()) return undefined;

        return allAssets().find(a => a.asset_identity === props.asset());
    });

    const handleAmountInput = (e: InputEvent) => {
        const raw = (e.target as HTMLInputElement).value;

        if (raw === "" || /^\d*\.?\d*$/.test(raw)) {
            props.onAmountChange(raw);
        }
    };

    const handleMax = () => {
        const balance = balanceHuman();

        if (balance) {
            props.onAmountChange(balance.value);
        }
    };

    return (
        <div class="space-y-1">
            <div class="input flex items-center gap-2 px-2 py-1 cursor-text" onClick={e => {
                const input = e.currentTarget.querySelector("input");

                if (input) input.focus();
            }}>
                <input
                  type="text"
                  class="flex-1 text-sm bg-transparent outline-none placeholder:text-muted tabular-nums"
                  placeholder={props.placeholder ?? "0.0"}
                  value={props.amount()}
                  onInput={handleAmountInput}
                  onClick={e => e.stopPropagation()}
                />
                <button
                  type="button"
                  class="flex items-center gap-1 px-2 py-0.5 rounded-md bg-surface-alt hover:bg-border transition-colors text-xs font-medium cursor-pointer shrink-0"
                  onClick={e => {
                        e.stopPropagation();
                        setIsOpen(true);
                    }}
                >
                    <Show
                      when={selectedAsset()}
                      fallback={<span class="text-muted">Asset</span>}
                    >
                        {asset => (
                            <>
                                <AssetIcon asset={asset()} class="size-4" />
                                <span>{asset().asset_symbol}</span>
                            </>
                        )}
                    </Show>
                    <FiChevronDown class="size-3 text-muted" />
                </button>
            </div>
            <Show when={balanceHuman()}>
                {balance => (
                    <div class="flex justify-between items-center text-xs text-muted px-1">
                        <span>
                            {props.balanceLabel ?? "Balance"}: {balance().display} {balance().symbol}
                        </span>
                        <button
                          type="button"
                          class="text-primary font-medium hover:underline cursor-pointer"
                          onClick={handleMax}
                        >
                            Max
                        </button>
                    </div>
                )}
            </Show>

            <Dialog open={isOpen()} onOpenChange={setIsOpen}>
                <Dialog.Portal>
                    <Dialog.Overlay class="fixed inset-0 z-50 bg-background/50" />
                    <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
                        <Dialog.Content class="bg-surface border border-border rounded-xl shadow-lg w-full max-w-md max-h-[80vh] flex flex-col outline-none">
                            <div class="flex items-center gap-2 border-b border-border px-4 py-3">
                                <FiSearch class="size-4 text-muted shrink-0" />
                                <input
                                  type="text"
                                  placeholder="Search assets..."
                                  class="flex-1 bg-transparent outline-none text-sm"
                                  value={searchQuery()}
                                  onInput={e => setSearchQuery(e.currentTarget.value)}
                                />
                                <Dialog.CloseButton class="text-muted hover:text-foreground cursor-pointer text-sm font-medium">
                                    Cancel
                                </Dialog.CloseButton>
                            </div>
                            <div class="flex-1 overflow-y-auto p-2 space-y-0.5">
                                <For each={assetGroups()}>
                                    {group => (
                                        <div>
                                            <Show when={group.label}>
                                                <div class="px-2 py-1.5 text-xs font-medium uppercase tracking-wide text-muted">
                                                    {group.label}
                                                </div>
                                            </Show>
                                            <For each={group.options}>
                                                {assetItem => {
                                                    const balanceInfo = createMemo(() => {
                                                        const bm = balancesMap();
                                                        const ab = bm?.get(assetItem.asset_identity);

                                                        if (!ab?.balance) return undefined;

                                                        try {
                                                            const bigint = BigInt(ab.balance);

                                                            return {
                                                                amountDisplay: formatAmount(bigint, { decimals: assetItem.asset_decimals, precision: 2 }),
                                                                quoteDisplay: ab.balance_quote
                                                                    ? formatAmount(BigInt(ab.balance_quote), { decimals: 6, precision: 2, currency: displayCurrency() })
                                                                    : undefined,
                                                            };
                                                        }
                                                        catch {
                                                            return undefined;
                                                        }
                                                    });

                                                    return (
                                                        <button
                                                          type="button"
                                                          class="flex items-center gap-3 w-full rounded-md px-3 py-2.5 text-sm hover:bg-surface-alt transition-colors cursor-pointer"
                                                          onClick={() => {
                                                                props.onAssetChange(assetItem.asset_identity);
                                                                setIsOpen(false);
                                                                setSearchQuery("");
                                                            }}
                                                        >
                                                            <AssetIcon asset={assetItem} class="size-8 shrink-0" />
                                                            <div class="flex-1 text-left min-w-0">
                                                                <div class="font-medium truncate">{assetItem.asset_name}</div>
                                                                <div class="text-muted text-xs">{assetItem.asset_symbol}</div>
                                                            </div>
                                                            <Show when={balanceInfo()}>
                                                                {info => (
                                                                    <div class="text-right tabular-nums">
                                                                        <Show when={info().quoteDisplay}>
                                                                            {quote => (
                                                                                <div class="text-sm">{quote()}</div>
                                                                            )}
                                                                        </Show>
                                                                        <div class="text-xs text-muted">{info().amountDisplay} {assetItem.asset_symbol}</div>
                                                                    </div>
                                                                )}
                                                            </Show>
                                                            <Show when={assetItem.asset_identity === props.asset() && !balanceInfo()}>
                                                                <FiCheck class="size-4 text-primary shrink-0" />
                                                            </Show>
                                                        </button>
                                                    );
                                                }}
                                            </For>
                                        </div>
                                    )}
                                </For>
                                <Show when={assetGroups().length === 0}>
                                    <div class="text-center text-muted text-sm py-8">No assets found</div>
                                </Show>
                            </div>
                        </Dialog.Content>
                    </div>
                </Dialog.Portal>
            </Dialog>
        </div>
    );
};
