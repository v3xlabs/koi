import { Component, Show, Suspense } from "solid-js";

import { useAssetQuote } from "#/api/asset";
import { useDisplayCurrency } from "#/api/context";
import { formatAmount } from "#/utils/units";

export const AssetQuote: Component<{ asset_identity: string; }> = ({ asset_identity }) => (
    <Suspense fallback={<div>Loading...</div>}>
        <AssetQuoteInner asset_identity={asset_identity} />
    </Suspense>
);

export const AssetQuoteInner: Component<{ asset_identity: string; }> = ({ asset_identity }) => {
    const quoteQuery = useAssetQuote(() => ({ path: { asset_identity } }));
    const { displayCurrency } = useDisplayCurrency();

    return (
        <Show when={quoteQuery.data}>
            {data => (
                <div class="text-nowrap tabular-nums" title={formatAmount(BigInt(data()), { precision: 2, decimals: 6, currency: displayCurrency() })}>
                    {formatAmount(BigInt(data()), { precision: 2, decimals: 6, notation: "compact", currency: displayCurrency() })}
                </div>
            )}
        </Show>
    );
};
