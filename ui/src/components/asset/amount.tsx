import { Accessor, Component, createMemo } from "solid-js";

import { useAsset } from "#/api/asset";

export type AssetAmountProperties = {
    amount: Accessor<bigint>;
    asset: Accessor<string>;
    decimals?: number;
};

export const AssetAmount: Component<AssetAmountProperties> = (props) => {
    const assetQuery = useAsset(() => ({ path: { asset_identity: props.asset() } }));
    const assetQueryData = createMemo(() => assetQuery.data);

    const decimals = props.decimals ?? 2;
    const symbol = createMemo(() => {
        const asset = props.asset();

        if (asset == "fiat:usd") {
            return "$";
        }

        return "?";
    });
    const denominator = createMemo(() => {
        const asset = props.asset();

        if (asset == "fiat:usd") {
            return 1e6;
        }

        const data = assetQueryData();

        if (data) {
            return data.asset_decimals;
        }

        return 1e6;
    });
    const formatted = createMemo(() => {
        const amount = props.amount();
        const denom = denominator();

        return (Number(amount) / denom).toFixed(decimals).split(".");
    });

    return (
        <div class="tabular-nums">
            <span class="text-foreground">
                {symbol()}
                {formatted()[0]}
            </span>
            <span class="text-muted">
                .
                {formatted()[1]
                    .padStart(2, "0")}
            </span>
        </div>
    );
};
