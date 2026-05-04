import { Component, Show, Suspense } from "solid-js";

import { useAssetQuote } from "#/api/asset";
import { formatUnits } from "#/utils/units";

export const AssetQuote: Component<{ asset_identity: string; }> = ({ asset_identity }) => (
    <Suspense fallback={<div>Loading...</div>}>
        <AssetQuoteInner asset_identity={asset_identity} />
    </Suspense>
);

export const AssetQuoteInner: Component<{ asset_identity: string; }> = ({ asset_identity }) => {
    const quoteQuery = useAssetQuote(() => ({ path: { asset_identity } }));

    return (
        <Show when={quoteQuery.data}>
            {data => (
                <div class="text-nowrap tabular-nums">
                    {formatUnits(BigInt(data()), 6, 2, "short")}
                </div>
            )}
        </Show>
    );
};
