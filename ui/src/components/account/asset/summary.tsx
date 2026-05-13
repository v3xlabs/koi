import { Skeleton } from "@kobalte/core/skeleton";
import { createQueries } from "@tanstack/solid-query";
import { Link } from "@tanstack/solid-router";
import { createColumnHelper, createSolidTable, flexRender, getCoreRowModel, getSortedRowModel, SortingState } from "@tanstack/solid-table";
import { FiChevronRight, FiChevronUp } from "solid-icons/fi";
import { Component, createMemo, createSignal, For, Suspense } from "solid-js";

import { useAccountAssets, useAccountBalances } from "#/api/account";
import { Asset, useAsset } from "#/api/asset";
import { useDisplayCurrency } from "#/api/context";
import { button } from "#/components/input/button";
import { formatAmount } from "#/utils/units";

import { AssetIcon } from "../../asset/icon";

type Data = { asset: Asset; price: bigint | undefined; balance: bigint | undefined; value: bigint | undefined; };
const helper = createColumnHelper<Data>();

const columns = [
    helper.accessor("asset.asset_name", {
        header: "Name",
        cell: ({ row }) => (
            <div class="flex items-center gap-2.5 py-3.5">
                <AssetIcon asset={row.original.asset} class="size-8" />
                <div>
                    <Skeleton visible={!row.original.asset.asset_name || row.original.asset.asset_name === "placeholder"} class="skeleton animate-spin">
                        {row.original.asset.asset_name}
                    </Skeleton>
                    <div class="text-muted">
                        <Skeleton visible={row.original.balance === undefined} class="skeleton">
                            <span class="tabular-nums">
                                {row.original.balance === undefined ? "-" : formatAmount(row.original.balance, { decimals: row.original.asset.asset_decimals, precision: 2, notation: "compact" })}
                            </span>
                            {" "}
                            {row.original.asset.asset_symbol}
                        </Skeleton>
                    </div>
                </div>
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
                    <span class="tabular-nums" title={row.original.value === undefined ? undefined : formatAmount(row.original.value, { precision: 2, decimals: 6, currency: displayCurrency() })}>
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
];

const AccountAssetSummaryInner: Component<{ account_identity: number; }> = ({ account_identity }) => {
    const accountAssetsQuery = useAccountAssets(() => ({ path: { account_identity } }));
    const assetQueries = createMemo(() => accountAssetsQuery.data?.map(asset_identity => useAsset.options({ path: { asset_identity } })) ?? []);

    const bulk = createQueries(() => ({
        queries: assetQueries(),
    }));

    const { displayCurrency } = useDisplayCurrency();
    const accountBalancesQuery = useAccountBalances(() => ({ path: { account_identity }, query: { display_currency: displayCurrency() } }));
    const balances = createMemo(() => accountBalancesQuery.data?.balances ?? []);

    const data = createMemo(() => bulk.flatMap((asset): Data[] => {
        if (!asset.data) return [];

        const balances2 = balances();
        const k = balances2.find(balance => balance.asset_identity === asset.data.asset_identity);

        return [{
            asset: asset.data,
            // price: price ? BigInt(price) : undefined,
            balance: k && k.balance ? BigInt(k.balance) : undefined,
            // value,
            price: k && k.asset_quote ? BigInt(k.asset_quote) : undefined,
            value: k && k.balance_quote ? BigInt(k.balance_quote) : undefined,
        }];
    })
        // filter anything below "1000 units", (this is a bit naive, but fine for these purposes)
        .filter(entry => entry.value && entry.value >= 1000n));

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
        <div class="bg-surface px-6 py-2.5 rounded-md w-full">
            <div class="flex justify-between items-center">
                <div class="text-sm text-muted font-bold">
                    Asset overview
                </div>
                <div class="flex justify-end">
                    <Link to="/acc/$account/assets" params={{ account: account_identity.toString() }} class={button({ variant: "ghost", class: "text-sm" })}>
                        View all
                        {" "}
                        <FiChevronRight class="size-3" />
                    </Link>
                </div>
            </div>
            <table class="w-full">
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
                                            {flexRender(
                                                cell.column.columnDef.cell,
                                                cell.getContext(),
                                            )}
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
    );
};

const AccountAssetSummarySkeleton: Component = () => (
    <div class="bg-surface px-6 py-2.5 rounded-md w-full">
        <div class="flex justify-between items-center">
        </div>
    </div>
);

export const AccountAssetSummary: Component<{ account_identity: number; }> = ({ account_identity }) => (
    <Suspense fallback={<AccountAssetSummarySkeleton />}>
        <AccountAssetSummaryInner account_identity={account_identity} />
    </Suspense>
);
