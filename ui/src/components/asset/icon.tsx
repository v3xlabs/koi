import { Component, Show } from "solid-js";

import { useAsset } from "#/api/asset";

export const AssetIcon: Component<{ asset_identity: string; }> = ({ asset_identity }) => {
    const assetQuery = useAsset(() => ({ path: { asset_identity } }));

    return (
        <Show when={assetQuery.data?.asset_icon_url}>
            {icon => <img src={icon()} alt={assetQuery.data?.asset_name} class="size-4 aspect-square rounded-full" />}
        </Show>
    );
};
