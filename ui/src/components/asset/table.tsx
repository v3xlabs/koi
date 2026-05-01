import { Skeleton } from "@kobalte/core/skeleton";
import { createColumnHelper, createSolidTable, flexRender, getCoreRowModel } from "@tanstack/solid-table";
import { Component, For } from "solid-js";

type Data = { name: string; price: bigint; balance: bigint; };
const helper = createColumnHelper<Data>();

const columns = [
    helper.accessor("name", {
        header: "Name",
        cell: ({ row }) => (
            <div class="flex items-center gap-2 py-3.5">
                <div class="size-10 bg-surface-alt border border-border rounded-full aspect-square" />
                <Skeleton visible={!row.original.name || row.original.name === "placeholder"} class="skeleton animate-spin">
                    {row.original.name}
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
                    {row.original.price?.toString()}
                </Skeleton>
            </div>
        ),
    }),
    helper.accessor("balance", {
        header: "Balance",
        cell: ({ row }) => (
            <div class="space-y-1">
                <Skeleton visible={!row.original.price} class="skeleton animate-spin">
                    {row.original.balance?.toString()}
                    {" "}
                    {row.original.name}
                </Skeleton>
                <Skeleton visible={!row.original.price || !row.original.balance} class="skeleton animate-spin text-muted">
                    $
                    {(Number(row.original.balance) * Number(row.original.price)).toFixed(2)}
                </Skeleton>
            </div>
        ),
    }),
];

export const AccountAssetTable: Component<{ account_identity: string; }> = (params) => {
    const data = [{
        name: "ETH",
        price: 1000n,
        balance: 100n,
    }, {
        name: "USDC",
        price: 1000n,
        balance: 100n,
    }, {
        name: "USDT",
        price: 1000n,
        balance: 100n,
    }, {
        name: "DAI",
        price: 1000n,
        balance: 100n,
    }, {
        name: "WBTC",
        price: 1000n,
        balance: 100n,
    },
    {
        name: "placeholder",
        price: undefined,
        balance: undefined,
    }];

    const table = createSolidTable({
        columns,
        get data() {
            return data;
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
