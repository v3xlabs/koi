import { Skeleton } from "@kobalte/core/skeleton";
import { createQueries } from "@tanstack/solid-query";
import { Link } from "@tanstack/solid-router";
import { createColumnHelper, createSolidTable, flexRender, getCoreRowModel, getSortedRowModel, SortingState } from "@tanstack/solid-table";
import { FiChevronRight, FiChevronUp } from "solid-icons/fi";
import { Accessor, Component, createEffect, createMemo, createSignal, For, Suspense } from "solid-js";

import { accountBalanceQuery, useAccountAssets, useAccountBalances } from "#/api/account";
import { Asset, useAsset } from "#/api/asset";
import { useDisplayCurrency, usePrivacyMode } from "#/api/context";
import { button } from "#/components/input/button";
import { privateAmount, privateAmountTitle } from "#/utils/privacy";
import { formatAmount, percentNumber } from "#/utils/units";

import { AssetIcon } from "../../asset/icon";

type Data = { asset: Asset; price: bigint | undefined; price_24h: bigint | undefined; balance: bigint | undefined; value: bigint | undefined; };
const helper = createColumnHelper<Data>();
const skeletonRows = Array.from({ length: 3 });
const skeletonBlock = {
    action: { width: 80, height: 32, radius: 6, class: "skeleton" },
    icon: { height: 32, circle: true, class: "skeleton" },
    name: { width: 96, height: 16, radius: 4, class: "skeleton" },
    symbol: { width: 64, height: 12, radius: 4, class: "skeleton" },
    value: { width: 80, height: 16, radius: 4, class: "skeleton" },
} as const;

const keepPreviousData: <T>(previousData: T | undefined) => T | undefined = previousData => previousData;

const SkeletonBlock: Component<{ variant: keyof typeof skeletonBlock; }> = props => (
    <Skeleton
      visible
      {...skeletonBlock[props.variant]}
    />
);

const createColumns = (quoteCurrency: Accessor<string>) => [
    helper.accessor("asset.asset_name", {
        header: "Name",
        cell: ({ row }) => {
            const { privacyMode } = usePrivacyMode();

            return (
                <div class="flex items-center gap-2.5 py-3.5">
                    <AssetIcon asset={row.original.asset} class="size-8" />
                    <div>
                        <Skeleton visible={!row.original.asset.asset_name || row.original.asset.asset_name === "placeholder"} class="skeleton animate-spin">
                            {row.original.asset.asset_name}
                        </Skeleton>
                        <div class="text-muted">
                            <Skeleton visible={row.original.balance === undefined} class="skeleton">
                                <span class="tabular-nums">
                                    {row.original.balance === undefined ? "-" : privateAmount(privacyMode(), formatAmount(row.original.balance, { decimals: row.original.asset.asset_decimals, precision: 2, notation: "compact" }))}
                                </span>
                                {" "}
                                {row.original.asset.asset_symbol}
                            </Skeleton>
                        </div>
                    </div>
                </div>
            );
        },
    }),
    helper.accessor("value", {
        header: "Value",
        cell: ({ row }) => {
            const { privacyMode } = usePrivacyMode();

            const percentageChange = row.original.price && row.original.price_24h ? percentNumber(row.original.price - row.original.price_24h, row.original.price_24h) : undefined;

            return (
                <div class="space-y-1 items-end flex flex-col justify-end">
                    <Skeleton visible={row.original.price === undefined || row.original.balance === undefined} class="skeleton max-w-24 max-h-4 text-end rounded-md">
                        <span class="tabular-nums" title={privateAmountTitle(privacyMode(), row.original.value === undefined ? undefined : formatAmount(row.original.value, { precision: 2, decimals: 6, currency: quoteCurrency() }))}>
                            {row.original.value === undefined ? "-" : privateAmount(privacyMode(), formatAmount(row.original.value, { precision: 2, decimals: 6, notation: "compact", currency: quoteCurrency() }))}
                        </span>
                    </Skeleton>
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
];

const AccountAssetSummaryInner: Component<{ account_identity: number; }> = ({ account_identity }) => {
    const accountAssetsQuery = useAccountAssets(() => ({ path: { account_identity } }));
    const assetQueries = createMemo(() => accountAssetsQuery.data?.map(asset_identity => useAsset.options({ path: { asset_identity } })) ?? []);

    const bulk = createQueries(() => ({
        queries: assetQueries(),
    }));

    const { displayCurrency } = useDisplayCurrency();
    const accountBalancesQuery = useAccountBalances(
        () => accountBalanceQuery(account_identity, displayCurrency()),
        { placeholderData: keepPreviousData },
    );
    const [quoteCurrency, setQuoteCurrency] = createSignal(displayCurrency());
    const balances = createMemo(() => accountBalancesQuery.data?.balances ?? []);

    createEffect(() => {
        if (accountBalancesQuery.data && !accountBalancesQuery.isPlaceholderData) {
            setQuoteCurrency(displayCurrency());
        }
    });

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
            price_24h: k && k.asset_24h_quote ? BigInt(k.asset_24h_quote) : undefined,
            value: k && k.balance_quote ? BigInt(k.balance_quote) : undefined,
        }];
    })
        // filter anything below "1000 units", (this is a bit naive, but fine for these purposes)
        .filter(entry => entry.value && entry.value >= 1000n));

    // eslint-disable-next-line no-restricted-syntax
    const [sorting, setSorting] = createSignal<SortingState>([{ id: "value", desc: false }]);

    const table = createSolidTable({
        columns: createColumns(quoteCurrency),
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
            <div class="-mx-2.5 overflow-x-clip">
                <table class="w-full">
                    <tbody class="w-full">
                        <For each={table.getRowModel().rows}>
                            {row => (
                                <>
                                    <tr class="group relative hover:bg-surface-alt rounded-2xl transition-colors w-full after:absolute after:bottom-0 after:left-2.5 after:right-2.5 after:h-px not-last:after:bg-border">
                                        <For each={row.getVisibleCells()}>
                                            {(cell, index) => (
                                                <td classList={{
                                                    "pl-5 -ml-2.5 -translate-x-2.5": index() === 0,
                                                    "pr-5 -mr-2.5 translate-x-2.5": index() === row.getVisibleCells().length - 1,
                                                }}
                                                >
                                                    {flexRender(
                                                        cell.column.columnDef.cell,
                                                        cell.getContext(),
                                                    )}
                                                </td>
                                            )}
                                        </For>
                                    </tr>
                                </>
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

const AccountAssetSummarySkeleton: Component = () => (
    <div class="bg-surface px-6 py-2.5 rounded-md w-full">
        <div class="flex justify-between items-center">
            <div class="text-sm text-muted font-bold">
                Asset overview
            </div>
            <SkeletonBlock variant="action" />
        </div>
        <div class="-mx-2.5 overflow-x-clip">
            <table class="w-full">
                <tbody class="w-full">
                    <For each={skeletonRows}>
                        {() => (
                            <tr class="relative after:absolute after:bottom-0 after:left-2.5 after:right-2.5 after:h-px not-last:after:bg-border">
                                <td class="pl-5 -ml-2.5 -translate-x-2.5">
                                    <div class="flex items-center gap-2.5 py-3.5">
                                        <SkeletonBlock variant="icon" />
                                        <div class="space-y-1.5">
                                            <SkeletonBlock variant="name" />
                                            <SkeletonBlock variant="symbol" />
                                        </div>
                                    </div>
                                </td>
                                <td class="pr-5 -mr-2.5 translate-x-2.5">
                                    <div class="flex justify-end">
                                        <SkeletonBlock variant="value" />
                                    </div>
                                </td>
                            </tr>
                        )}
                    </For>
                </tbody>
            </table>
        </div>
    </div>
);

export const AccountAssetSummary: Component<{ account_identity: number; }> = ({ account_identity }) => (
    <Suspense fallback={<AccountAssetSummarySkeleton />}>
        <AccountAssetSummaryInner account_identity={account_identity} />
    </Suspense>
);
