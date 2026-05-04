import { Skeleton } from "@kobalte/core/skeleton";
import { createQueries } from "@tanstack/solid-query";
import { createColumnHelper, createSolidTable, flexRender, getCoreRowModel, getSortedRowModel, SortingState } from "@tanstack/solid-table";
import { Component, createMemo, createSignal, For, Suspense } from "solid-js";

import { useAccountAssets, useAccountBalances } from "#/api/account";
import { Asset, useAsset } from "#/api/asset";
import { formatUnits } from "#/utils/units";

import { AssetIcon } from "./icon";

type Data = { asset: Asset; price: bigint | undefined; balance: bigint | undefined; value: bigint | undefined; };
const helper = createColumnHelper<Data>();

const columns = [
    helper.accessor("asset.asset_name", {
        header: "Name",
        cell: ({ row }) => (
            <div class="flex items-center gap-2 py-3.5">
                <AssetIcon asset={row.original.asset} />
                <Skeleton visible={!row.original.asset.asset_name || row.original.asset.asset_name === "placeholder"} class="skeleton animate-spin">
                    {row.original.asset.asset_name}
                </Skeleton>
            </div>
        ),
    }),
    helper.accessor("price", {
        header: "Price",
        cell: ({ row }) => (
            <div class="flex items-center gap-2 py-3.5">
                <span class="min-w-4">
                    <Skeleton visible={!row.original.price} class="skeleton">
                        $
                        <span class="tabular-nums">
                            {row.original.price ? formatUnits(row.original.price, 6, 2, "short") : "-"}
                        </span>
                    </Skeleton>
                </span>
            </div>
        ),
    }),
    helper.accessor("value", {
        header: "Balance",
        cell: ({ row }) => (
            <div class="space-y-1">
                <Skeleton visible={row.original.balance === undefined} class="skeleton">
                    <span class="tabular-nums">
                        {row.original.balance === undefined ? "-" : formatUnits(row.original.balance, row.original.asset.asset_decimals, 2, "short")}
                    </span>
                    {" "}
                    {row.original.asset.asset_symbol}
                </Skeleton>
                <Skeleton visible={row.original.price === undefined || row.original.balance === undefined} class="skeleton text-muted max-w-24 max-h-4 rounded-md">
                    $
                    <span class="tabular-nums">
                        {row.original.value === undefined ? "-" : formatUnits(row.original.value, 6, 2, "short")}
                    </span>
                </Skeleton>
            </div>
        ),
        sortingFn: (rowA, rowB) => {
            const valueA = rowA.original.value ?? 0n;
            const valueB = rowB.original.value ?? 0n;

            return valueA > valueB ? -1 : 0;
        },
    }),
];

const AccountAssetTableInner: Component<{ account_identity: number; }> = ({ account_identity }) => {
    const accountAssetsQuery = useAccountAssets(() => ({ path: { account_identity } }));
    const assetQueries = createMemo(() => accountAssetsQuery.data?.map(asset_identity => useAsset.options({ path: { asset_identity } })) ?? []);

    const bulk = createQueries(() => ({
        queries: assetQueries(),
    }));

    const accountBalancesQuery = useAccountBalances(() => ({ path: { account_identity } }));
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
    }));

    const total = createMemo(() => data().reduce((acc, curr) => (curr.value ? acc + curr.value : acc), 0n));

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
                <div class="text-sm text-muted">
                    Total: $
                    <span class="tabular-nums">
                        {formatUnits(total(), 6, 2, "short")}
                    </span>
                </div>
            </div>
            <table class="w-full">
                <thead class="border-b border-border">
                    <For each={table.getHeaderGroups()}>
                        {headerGroup => (
                            <tr>
                                <For each={headerGroup.headers}>
                                    {header => (
                                        <th class="text-left pb-2.5 py-0.5">
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
