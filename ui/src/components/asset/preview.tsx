import { Component, Suspense } from "solid-js";

import { Asset, useAsset } from "#/api/asset";

import { AssetIcon } from "./icon";

export type AssetPreviewProperties = { asset_identity: string; } | { asset: Asset; };

export const AssetPreview: Component<{ asset_identity: string; }> = ({ asset_identity }) => (
    <Suspense fallback={<div>Loading...</div>}>
        <AssetPreviewInner asset_identity={asset_identity} />
    </Suspense>
);

const AssetPreviewInner: Component<{ asset_identity: string; }> = ({ asset_identity }) => {
    const assetQuery = useAsset(() => ({ path: { asset_identity } }));

    return (
        <div class="w-full flex items-center gap-2">
            <AssetIcon asset_identity={asset_identity} />
            {assetQuery.data?.asset_name}
        </div>
    );
};
