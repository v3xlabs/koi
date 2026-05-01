import { Skeleton } from "@kobalte/core/skeleton";
import { createQueries } from "@tanstack/solid-query";
import { createColumnHelper, createSolidTable, flexRender, getCoreRowModel } from "@tanstack/solid-table";
import { Component, createMemo, For } from "solid-js";

import { api } from "#/api";
import { useAccountAssets } from "#/api/account";
import { Asset } from "#/api/asset";

import { AssetIcon } from "./icon";

type Data = { asset: Asset; price: bigint; balance: bigint; };
const helper = createColumnHelper<Data>();

const columns = [
    helper.accessor("asset.asset_name", {
        header: "Name",
        cell: ({ row }) => (
            <div class="flex items-center gap-2 py-3.5">
                <AssetIcon asset_identity={row.original.asset.asset_identity} />
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
                <Skeleton visible={!row.original.price} class="skeleton animate-spin">
                    $
                    <span class="tabular-nums">
                        {row.original.price?.toString()}
                    </span>
                </Skeleton>
            </div>
        ),
    }),
    helper.accessor("balance", {
        header: "Balance",
        cell: ({ row }) => (
            <div class="space-y-1">
                <Skeleton visible={!row.original.price} class="skeleton animate-spin">
                    <span class="tabular-nums">
                        {row.original.balance?.toString()}
                    </span>
                    {" "}
                    {row.original.asset.asset_symbol}
                </Skeleton>
                <Skeleton visible={!row.original.price || !row.original.balance} class="skeleton animate-spin text-muted">
                    $
                    <span class="tabular-nums">
                        {(Number(row.original.balance) * Number(row.original.price)).toFixed(2)}
                    </span>
                </Skeleton>
            </div>
        ),
    }),
];

export const AccountAssetTable: Component<{ account_identity: number; }> = ({ account_identity }) => {
    const accountAssetsQuery = useAccountAssets(() => ({ path: { account_identity } }));

    const bulk = createQueries(() => ({
        queries: accountAssetsQuery.data?.map(asset => ({
            queryKey: ["asset", asset],
            queryFn: async () => {
                const request = await api("/asset/{asset_identity}", "get", {
                    path: {
                        asset_identity: asset,
                    },
                });

                return request.data;
            },
        })) ?? [],
    }));

    const data = createMemo(() => bulk?.map(asset => ({
        asset: asset.data,
        price: 1000n,
        balance: 100n,
    }))?.filter(asset => asset.asset !== undefined) ?? []);

    // const data = [{
    //     name: "ETH",
    //     price: 1000n,
    //     balance: 100n,
    // }, {
    //     name: "USDC",
    //     price: 1000n,
    //     balance: 100n,
    // }, {
    //     name: "USDT",
    //     price: 1000n,
    //     balance: 100n,
    // }, {
    //     name: "DAI",
    //     price: 1000n,
    //     balance: 100n,
    // }, {
    //     name: "WBTC",
    //     price: 1000n,
    //     balance: 100n,
    // },
    // {
    //     name: "placeholder",
    //     price: undefined,
    //     balance: undefined,
    // }];

    const table = createSolidTable({
        columns,
        get data() {
            return data();
        },
        getCoreRowModel: getCoreRowModel(),
    });

    return (
        <div class="bg-surface px-6 py-2.5 rounded-md w-full">
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
