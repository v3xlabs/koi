import { Skeleton } from "@kobalte/core/skeleton";
import { createQueries } from "@tanstack/solid-query";
import { createColumnHelper, createSolidTable, flexRender, getCoreRowModel, getSortedRowModel, SortingState } from "@tanstack/solid-table";
import { FaSolidRefresh } from "solid-icons/fa";
import { FiArrowUpRight, FiChevronUp } from "solid-icons/fi";
import { Component, createMemo, createSignal, For, Show, Suspense } from "solid-js";

import { useAccountAssets, useAccountBalances } from "#/api/account";
import { Asset, useAsset } from "#/api/asset";
import { useDisplayCurrency } from "#/api/context";
import { AssetAmount } from "#/components/asset/amount";
import { button } from "#/components/input/button";
import { DisplayCurrencySelector } from "#/components/quoter/display";
import { formatAmount, percentNumber } from "#/utils/units";

import { AssetIcon } from "../../asset/icon";

type Data = { asset: Asset; price: bigint | undefined; balance: bigint | undefined; value: bigint | undefined; weight: number | undefined; };
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

            return (
                <div class="flex items-center gap-2 py-3.5">
                    <span class="min-w-4">
                        <Skeleton visible={row.original.price === undefined} class="skeleton">
                            <span class="tabular-nums" title={row.original.price === undefined ? undefined : formatAmount(row.original.price, { decimals: 6, precision: 2, currency: displayCurrency() })}>
                                {row.original.price === undefined ? "-" : formatAmount(row.original.price, { decimals: 6, precision: 2, notation: "compact", currency: displayCurrency() })}
                            </span>
                        </Skeleton>
                    </span>
                </div>
            );
        },
    }),
    helper.accessor("value", {
        header: "Balance",
        cell: ({ row }) => (
            <div class="space-y-1">
                <Skeleton visible={row.original.balance === undefined} class="skeleton">
                    <span class="tabular-nums" title={row.original.balance === undefined ? undefined : formatAmount(row.original.balance, { decimals: row.original.asset.asset_decimals })}>
                        {row.original.balance === undefined ? "-" : formatAmount(row.original.balance, { decimals: row.original.asset.asset_decimals, notation: "compact" })}
                    </span>
                </Skeleton>
            </div>
        ),
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

            return (
                <div class="space-y-1 items-end flex flex-col justify-end">
                    <Skeleton visible={row.original.price === undefined || row.original.balance === undefined} class="skeleton max-w-24 max-h-4 text-end rounded-md">
                        <span class="tabular-nums">
                            {row.original.value === undefined ? "-" : formatAmount(row.original.value, { precision: 2, decimals: 6, notation: "compact", currency: displayCurrency() })}
                        </span>
                    </Skeleton>
                    <div class="flex items-center gap-0.5">
                        <FiChevronUp class="size-3" />
                        <span class="text-muted text-xs">
                            0.00%
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

    const accountBalancesQuery = useAccountBalances(() => ({ path: { account_identity }, query: { display_currency: displayCurrency() } }));
    const balances = createMemo(() => accountBalancesQuery.data?.balances ?? []);

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
                <div class="flex flex-col items-end justify-center gap-2">
                    <div class="text-muted text-sm flex items-center gap-2">
                        <Suspense>
                            <span>
                                Updated
                                {" "}
                                {accountBalancesQuery.data?.updated_at ? Date.parse(accountBalancesQuery.data.updated_at).toLocaleString() : "-"}
                            </span>
                        </Suspense>
                        <Show when={!accountBalancesQuery.isLoading}>
                            <button
                              class={button({ variant: "ghost", size: "small", square: true })}
                              onClick={() => accountBalancesQuery.refetch()}
                            >
                                <FaSolidRefresh classList={{
                                    "size-3.5": true,
                                    "animate-spin": accountBalancesQuery.isRefetching,
                                }}
                                />
                            </button>
                        </Show>
                    </div>
                    <div class="flex items-center gap-2 justify-end">
                        <button class={button({ variant: "outline", class: "text-sm" })}>
                            Manage assets
                        </button>
                        <DisplayCurrencySelector />
                    </div>
                </div>
            </div>
            <div class="bg-surface px-6 py-2.5 rounded-md w-full">
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
                    <tbody class="divide-y divide-border">
                        <For each={table.getRowModel().rows}>
                            {row => (
                                <tr class="relative group z-10">
                                    <For each={row.getVisibleCells()}>
                                        {(cell, index) => (
                                            <td>
                                                {
                                                    index() === 0
                                                    && <div class="group-hover:-inset-x-2.5 group-hover:opacity-100 opacity-0 transition-all -z-10 absolute inset-y-0 inset-x-0 bg-surface-alt rounded-md">            </div>
                                                }
                                                <div
                                                  classList={{
                                                        "text-left": index() === 0,
                                                        "text-right flex justify-end": index() !== 0,
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
