import { Component, Show, Suspense } from "solid-js";

import { Asset, useAsset } from "#/api/asset";
import { narrow } from "#/utils/narrow";

import { AssetIcon } from "./icon";

export type AssetPreviewProperties = { asset_identity: string; } | { asset: Asset; };

export const AssetPreview: Component<AssetPreviewProperties> = props => (
    <Suspense fallback={<div>Loading...</div>}>
        <Show when={narrow(() => props, x => "asset_identity" in x)}>
            {asset_identity => <AssetPreviewInner asset_identity={asset_identity().asset_identity} />}
        </Show>
        <Show when={narrow(() => props, x => "asset" in x)}>
            {asset => <AssetPreviewInnerWithAsset asset={asset().asset} />}
        </Show>
    </Suspense>
);

const AssetPreviewInner: Component<{ asset_identity: string; }> = (props) => {
    const assetQuery = useAsset(() => ({ path: { asset_identity: props.asset_identity } }));

    return (
        <Show when={assetQuery.data}>
            {asset => <AssetPreviewInnerWithAsset asset={asset()} />}
        </Show>
    );
};

const AssetPreviewInnerWithAsset: Component<{ asset: Asset; }> = props => (
    <div class="inline-flex items-center gap-2">
        <AssetIcon asset_identity={props.asset?.asset_identity} />
        {props.asset?.asset_name}
    </div>
);
