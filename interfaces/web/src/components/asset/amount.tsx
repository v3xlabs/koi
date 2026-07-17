import { Accessor, Component, createMemo, Show } from "solid-js";

import { useAsset } from "#/api/asset";
import { usePrivacyMode } from "#/api/context";
import { PRIVATE_AMOUNT } from "#/utils/privacy";
import { formatAmountParts } from "#/utils/units";

export type AssetAmountProperties = {
    amount: Accessor<bigint>;
    asset: Accessor<string>;
    decimals?: number;
};

export const AssetAmount: Component<AssetAmountProperties> = (props) => {
    const assetQuery = useAsset(() => ({ path: { asset_identity: props.asset() } }));
    const assetQueryData = createMemo(() => assetQuery.data);
    const { privacyMode } = usePrivacyMode();

    const parts = createMemo(() => formatAmountParts(props.amount(), {
        decimals: assetQueryData()?.asset_decimals ?? 2,
        precision: props.decimals ?? 2,
        currency: props.asset().startsWith("fiat:") ? props.asset() : undefined,
    }),
    );

    const suffix = createMemo(() => {
        if (props.asset().startsWith("fiat:")) return undefined;

        return assetQueryData()?.asset_symbol;
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
                    <Show
                      when={!privacyMode()}
                      fallback={<span class="text-foreground">{PRIVATE_AMOUNT}</span>}
                    >
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
                    </Show>
                )}
            </Show>
            <Show when={suffix()}>
                {suffix => (
                    <span class="text-foreground">
                        {" " + suffix()}
                    </span>
                )}
            </Show>
        </div>
    );
};
