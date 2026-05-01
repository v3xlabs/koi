import { Component } from "solid-js";

import { useAsset } from "#/api/asset";

import { AssetIcon } from "./icon";

export const AssetPreview: Component<{ asset_identity: string; }> = ({ asset_identity }) => {
    const assetQuery = useAsset(() => ({ path: { asset_identity } }));

    return (
        <div class="w-full flex items-center gap-2">
            <AssetIcon asset_identity={asset_identity} />
            {assetQuery.data?.asset_name}
        </div>
    );
};
