import { Skeleton } from "@kobalte/core/skeleton";
import { createQueries } from "@tanstack/solid-query";
import { createColumnHelper, createSolidTable, flexRender, getCoreRowModel, getSortedRowModel, SortingState } from "@tanstack/solid-table";
import { FaSolidRefresh } from "solid-icons/fa";
import { FiArrowUpRight, FiChevronUp, FiPlus } from "solid-icons/fi";
import { Component, createMemo, createSignal, For, Show, Suspense } from "solid-js";

import { accountBalanceQuery, refreshAccountBalances, useAccountAssets, useAccountBalances } from "#/api/account";
import { Asset, useAsset } from "#/api/asset";
import { useDisplayCurrency, usePrivacyMode } from "#/api/context";
import { AssetAdd } from "#/components/asset/add";
import { AssetAmount } from "#/components/asset/amount";
import { button } from "#/components/input/button";
import { DisplayCurrencySelector } from "#/components/quoter/display";
import { FormattedTime } from "#/components/time";
import { privateAmount, privateAmountTitle } from "#/utils/privacy";
import { formatAmount, percentNumber } from "#/utils/units";

import { AssetIcon } from "../../asset/icon";
import { AccountAssetManage } from "./manage";

type Data = { asset: Asset; price: bigint | undefined; price_24h: bigint | undefined; balance: bigint | undefined; value: bigint | undefined; weight: number | undefined; };
const helper = createColumnHelper<Data>();

const columns = [
    helper.accessor("asset.asset_name", {
        header: "Name",
        cell: ({ row }) => (
            <div class="flex items-center gap-3 py-3.5">
                <AssetIcon asset={row.original.asset} class="size-8" />
                <div>
                    <Skeleton visible={!row.original.asset.asset_name || row.original.asset.asset_name === "placeholder"} class="skeleton animate-spin">
                        {row.original.asset.asset_name}
                    </Skeleton>
                    <Skeleton visible={row.original.asset.asset_symbol === undefined} class="skeleton text-muted text-sm">
                        {row.original.asset.asset_symbol}
                    </Skeleton>
                </div>
            </div>
        ),
    }),
    helper.accessor("price", {
        header: "Price",
        cell: ({ row }) => {
            const { displayCurrency } = useDisplayCurrency();
            const { privacyMode } = usePrivacyMode();

            return (
                <div class="flex items-center gap-2 py-3.5">
                    <span class="min-w-4">
                        <Show
                          when={row.original.price !== undefined}
                          fallback={(
                                <span class="text-muted">
                                    ---
                                </span>
                            )}
                        >
                            <span class="tabular-nums" title={privateAmountTitle(privacyMode(), row.original.price === undefined ? undefined : formatAmount(row.original.price, { decimals: 6, precision: 2, currency: displayCurrency() }))}>
                                {row.original.price === undefined ? "-" : privateAmount(privacyMode(), formatAmount(row.original.price, { decimals: 6, precision: 2, notation: "compact", currency: displayCurrency() }))}
                            </span>
                        </Show>
                    </span>
                </div>
            );
        },
    }),
    helper.accessor("value", {
        header: "Balance",
        cell: ({ row }) => {
            const { privacyMode } = usePrivacyMode();

            return (
                <div class="space-y-1">
                    <Skeleton visible={row.original.balance === undefined} class="skeleton">
                        <span class="tabular-nums" title={privateAmountTitle(privacyMode(), row.original.balance === undefined ? undefined : formatAmount(row.original.balance, { decimals: row.original.asset.asset_decimals }))}>
                            {row.original.balance === undefined ? "-" : privateAmount(privacyMode(), formatAmount(row.original.balance, { decimals: row.original.asset.asset_decimals, notation: "compact" }))}
                        </span>
                    </Skeleton>
                </div>
            );
        },
        sortingFn: (rowA, rowB) => {
            const valueA = rowA.original.balance ?? 0n;
            const valueB = rowB.original.balance ?? 0n;

            return valueA > valueB ? -1 : 0;
        },
    }),
    helper.accessor("weight", {
        header: "Weight",
        cell: ({ row }) => (
            <div class="space-y-1">
                <Skeleton visible={row.original.weight === undefined} class="skeleton">
                    {row.original.weight === undefined ? "-" : `${row.original.weight}%`}
                </Skeleton>
            </div>
        ),
    }),
    helper.accessor("value", {
        header: "Value",
        cell: ({ row }) => {
            const { displayCurrency } = useDisplayCurrency();
            const { privacyMode } = usePrivacyMode();

            const percentageChange = row.original.price && row.original.price_24h ? percentNumber(row.original.price - row.original.price_24h, row.original.price_24h) : undefined;

            return (
                <div class="space-y-1 items-end flex flex-col justify-end">
                    {/* <Skeleton visible={row.original.price === undefined || row.original.balance === undefined} class="skeleton max-w-24 max-h-4 text-end rounded-md"> */}
                    <Show
                      when={row.original.value !== undefined}
                      fallback={(
                            <span>
                                -
                            </span>
                        )}
                    >
                        <span class="tabular-nums">
                            {row.original.value === undefined ? "-" : privateAmount(privacyMode(), formatAmount(row.original.value, { precision: 2, decimals: 6, notation: "compact", currency: displayCurrency() }))}
                        </span>
                    </Show>
                    {/* </Skeleton> */}
                    <div class="">
                        <span classList={{
                            "text-xs flex items-center gap-0.5": true,
                            "text-muted": percentageChange == 0,
                            "text-[#008000]": !!(percentageChange && percentageChange > 0),
                            "text-[#FF0000]": !!(percentageChange && percentageChange < 0),
                        }}
                        >
                            <FiChevronUp classList={{
                                "size-3": true,
                                "rotate-180": !!(percentageChange && percentageChange < 0),
                            }}
                            />
                            {percentageChange}
                            %
                        </span>
                    </div>
                </div>
            );
        },
        sortingFn: (rowA, rowB) => {
            const valueA = rowA.original.value ?? 0n;
            const valueB = rowB.original.value ?? 0n;

            return valueA > valueB ? -1 : 0;
        },
    }),
    helper.display({
        header: "Actions",
        cell: () => (
            <div class="flex items-center gap-2 py-3.5">
                <button class={button({ variant: "secondary", size: "small", square: true })}>
                    <FiArrowUpRight class="size-3" />
                </button>
            </div>
        ),
    }),
];

const AccountAssetTableInner: Component<{ account_identity: number; }> = ({ account_identity }) => {
    const { displayCurrency } = useDisplayCurrency();
    const accountAssetsQuery = useAccountAssets(() => ({ path: { account_identity } }));
    const assetQueries = createMemo(() => accountAssetsQuery.data?.map(asset_identity => useAsset.options({ path: { asset_identity } })) ?? []);

    const bulk = createQueries(() => ({
        queries: assetQueries(),
    }));

    const accountBalancesQuery = useAccountBalances(() => accountBalanceQuery(account_identity, displayCurrency()));
    const balances = createMemo(() => accountBalancesQuery.data?.balances ?? []);
    const [refreshingBalances, setRefreshingBalances] = createSignal(false);
    const [manageOpen, setManageOpen] = createSignal(false);
    const [addOpen, setAddOpen] = createSignal(false);
    const [afterAddAction, setAfterAddAction] = createSignal<"none" | "back">("none");

    const beginAddAssetFromManage = () => {
        setAfterAddAction("back");
        setManageOpen(false);
        setAddOpen(true);
    };

    const onAssetCreated = () => {
        setAddOpen(false);

        if (afterAddAction() === "back") {
            setManageOpen(true);
        }

        setAfterAddAction("none");
    };

    const onAddOpenChange = (open: boolean) => {
        setAddOpen(open);

        if (!open) {
            if (afterAddAction() === "back") {
                setManageOpen(true);
            }

            setAfterAddAction("none");
        }
    };

    const refreshBalances = async () => {
        setRefreshingBalances(true);

        try {
            await refreshAccountBalances({
                path: { account_identity },
                query: { display_currency: displayCurrency() },
            });
        }
        finally {
            setRefreshingBalances(false);
        }
    };

    const totalValue = createMemo(() => BigInt(accountBalancesQuery.data?.total_quote ?? 0));

    const data = createMemo(() => bulk.flatMap((asset): Data[] => {
        if (!asset.data) return [];

        const balances2 = balances();
        const k = balances2.find(balance => balance.asset_identity === asset.data.asset_identity);
        const weight = k && k.balance_quote ? percentNumber(BigInt(k.balance_quote), totalValue()) : undefined;

        return [{
            asset: asset.data,
            balance: k && k.balance ? BigInt(k.balance) : undefined,
            price: k && k.asset_quote ? BigInt(k.asset_quote) : undefined,
            price_24h: k && k.asset_24h_quote ? BigInt(k.asset_24h_quote) : undefined,
            value: k && k.balance_quote ? BigInt(k.balance_quote) : undefined,
            weight,
        }];
    }));

    // eslint-disable-next-line no-restricted-syntax
    const [sorting, setSorting] = createSignal<SortingState>([{ id: "value", desc: false }]);

    const table = createSolidTable({
        columns,
        get data() {
            return data();
        },
        state: {
            sorting: sorting(),
        },
        getSortedRowModel: getSortedRowModel(),
        getCoreRowModel: getCoreRowModel(),
        onSortingChange: setSorting,
    });

    return (
        <div class="w-full space-y-4">
            <AssetAdd
              open={addOpen()}
              onOpenChange={onAddOpenChange}
              onSuccess={onAssetCreated}
            />
            <div class="flex justify-between items-center">
                <div>
                    <div class="text-sm text-muted font-bold">Total assets value</div>
                    <div class="text-2xl">
                        <AssetAmount
                          amount={() => totalValue()}
                          asset={displayCurrency}
                        />
                    </div>
                </div>
                <div class="">
                    <div class="flex items-center gap-2 justify-end">
                        <div class="text-muted text-sm flex items-center gap-2">
                            <Suspense>
                                <span class="">
                                    <FormattedTime value={accountBalancesQuery.data?.updated_at} prefix="Updated " />
                                </span>
                            </Suspense>
                            <Show when={!accountBalancesQuery.isLoading || accountBalancesQuery.data}>
                                <button
                                  class={button({ variant: "ghost", size: "small", square: true })}
                                  onClick={() => { void refreshBalances(); }}
                                >
                                    <FaSolidRefresh classList={{
                                        "size-3.5": true,
                                        "animate-spin": refreshingBalances(),
                                    }}
                                    />
                                </button>
                            </Show>
                        </div>
                        <AccountAssetManage
                          account_identity={account_identity}
                          open={manageOpen()}
                          onOpenChange={setManageOpen}
                          onAddAsset={beginAddAssetFromManage}
                        />
                        <DisplayCurrencySelector />
                    </div>
                </div>
            </div>
            <div class="bg-surface px-4 py-2.5 rounded-md w-full">
                <table class="w-full">
                    <thead class="border-b border-border">
                        <For each={table.getHeaderGroups()}>
                            {headerGroup => (
                                <tr>
                                    <For each={headerGroup.headers}>
                                        {(header, index) => (
                                            <th classList={{
                                                "pb-2.5 py-0.5": true,
                                                "text-left": index() === 0,
                                                "text-right": index() !== 0,
                                            }}
                                            >
                                                {header.isPlaceholder
                                                    ? null
                                                    : flexRender(
                                                        header.column.columnDef.header,
                                                        header.getContext(),
                                                    )}
                                            </th>
                                        )}
                                    </For>
                                </tr>
                            )}
                        </For>
                    </thead>
                    <tbody class="">
                        <For each={table.getRowModel().rows}>
                            {row => (
                                <tr class="group relative hover:bg-surface-alt rounded-2xl transition-colors w-full after:absolute after:bottom-0 after:left-2.5 after:right-2.5 after:h-px not-last:after:bg-border">
                                    <For each={row.getVisibleCells()}>
                                        {(cell, index) => (
                                            <td
                                              classList={{
                                                    "pl-5 -ml-2.5 -translate-x-2.5": index() === 0,
                                                    "pr-5 -mr-2.5 translate-x-2.5": index() === row.getVisibleCells().length - 1,
                                                }}
                                            >
                                                <div
                                                  classList={{
                                                        "text-left": index() === 0,
                                                        "text-right flex justify-end": index() !== 0,
                                                        "relative z-10": true,
                                                    }}
                                                >
                                                    {flexRender(
                                                        cell.column.columnDef.cell,
                                                        cell.getContext(),
                                                    )}
                                                </div>
                                            </td>
                                        )}
                                    </For>
                                </tr>
                            )}
                        </For>
                        <Show when={table.getRowModel().rows.length === 0}>
                            <tr
                              class="group relative hover:bg-surface-alt rounded-2xl transition-colors w-full cursor-pointer"
                              onClick={() => setManageOpen(true)}
                            >
                                <td
                                  colspan={table.getVisibleLeafColumns().length}
                                  class="pl-5 py-3.5"
                                >
                                    <div class="flex items-center gap-3 text-muted">
                                        <div class="size-8 rounded-full border border-dashed border-border flex items-center justify-center">
                                            <FiPlus class="size-4" />
                                        </div>
                                        Add a new asset
                                    </div>
                                </td>
                            </tr>
                        </Show>
                    </tbody>
                    <tfoot>
                        <For each={table.getFooterGroups()}>
                            {footerGroup => (
                                <tr>
                                    <For each={footerGroup.headers}>
                                        {header => (
                                            <th>
                                                {header.isPlaceholder
                                                    ? null
                                                    : flexRender(
                                                        header.column.columnDef.footer,
                                                        header.getContext(),
                                                    )}
                                            </th>
                                        )}
                                    </For>
                                </tr>
                            )}
                        </For>
                    </tfoot>
                </table>
            </div>
        </div>
    );
};

const AccountAssetTableSkeleton: Component = () => (
    <div class="bg-surface px-6 py-2.5 rounded-md w-full">
        <div class="flex justify-between items-center">
        </div>
    </div>
);

export const AccountAssetTable: Component<{ account_identity: number; }> = ({ account_identity }) => (
    <Suspense fallback={<AccountAssetTableSkeleton />}>
        <AccountAssetTableInner account_identity={account_identity} />
    </Suspense>
);
