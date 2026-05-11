import { Accessor, Component, createMemo, Show } from "solid-js";

import { useAsset } from "#/api/asset";
import { formatAmountParts } from "#/utils/units";

export type AssetAmountProperties = {
    amount: Accessor<bigint>;
    asset: Accessor<string>;
    decimals?: number;
};

export const AssetAmount: Component<AssetAmountProperties> = (props) => {
    const assetQuery = useAsset(() => ({ path: { asset_identity: props.asset() } }));
    const assetQueryData = createMemo(() => assetQuery.data);

    const precision = props.decimals ?? 2;
    const scale = createMemo(() => {
        const asset = props.asset();

        if (asset == "fiat:usd") {
            return 6;
        }

        return assetQueryData()?.asset_decimals;
    });
    const parts = createMemo(() => {
        const decimals = scale();

        if (decimals === undefined) return undefined;

        return formatAmountParts(props.amount(), {
            decimals,
            precision,
            style: props.asset() === "fiat:usd" ? "currency" : "decimal",
        });
    });

    return (
        <div class="tabular-nums">
            <Show
              when={parts()}
              fallback={(
                <span class="text-muted">
                        ...
                </span>
              )}
            >
                {formatted => (
                    <>
                        <span class="text-foreground">
                            {formatted().prefix}
                            {formatted().integer}
                        </span>
                        {formatted().fraction && (
                            <span class="text-muted">
                                {formatted().decimal}
                                {formatted().fraction}
                            </span>
                        )}
                        {formatted().suffix && (
                            <span class="text-foreground">
                                {formatted().suffix}
                            </span>
                        )}
                    </>
                )}
            </Show>
        </div>
    );
};
