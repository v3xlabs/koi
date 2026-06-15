import { Component, Show, Suspense } from "solid-js";

import { useAssetQuote } from "#/api/asset";
import { useDisplayCurrency, usePrivacyMode } from "#/api/context";
import { privateAmount, privateAmountTitle } from "#/utils/privacy";
import { formatAmount } from "#/utils/units";

export const AssetQuote: Component<{ asset_identity: string; }> = ({ asset_identity }) => (
    <Suspense fallback={<div>Loading...</div>}>
        <AssetQuoteInner asset_identity={asset_identity} />
    </Suspense>
);

export const AssetQuoteInner: Component<{ asset_identity: string; }> = ({ asset_identity }) => {
    const quoteQuery = useAssetQuote(() => ({ path: { asset_identity } }), {throwOnError: false});
    const { displayCurrency } = useDisplayCurrency();
    const { privacyMode } = usePrivacyMode();

    return (
        <Show when={quoteQuery.data} fallback={<div>---</div>}>
            {data => (
                <div class="text-nowrap tabular-nums" title={privateAmountTitle(privacyMode(), formatAmount(BigInt(data()), { precision: 2, decimals: 6, currency: displayCurrency() }))}>
                    {privateAmount(privacyMode(), formatAmount(BigInt(data()), { precision: 2, decimals: 6, notation: "compact", currency: displayCurrency() }))}
                </div>
            )}
        </Show>
    );
};
