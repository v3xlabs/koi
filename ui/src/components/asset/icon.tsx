import { Component, createMemo, Show } from "solid-js";

import { Asset, useAsset } from "#/api/asset";

type AssetIconProps = { asset: Asset; } | { asset_identity: string; };

export const AssetIconImage: Component<{ asset?: Asset; }> = props => (
    <Show when={props.asset?.asset_icon_url}>
        {icon => <img src={icon()} alt={props.asset?.asset_name} class="size-4 aspect-square rounded-full" />}
    </Show>
);

export const AssetIcon: Component<AssetIconProps> = (props) => {
    const assetQuery = "asset_identity" in props ? useAsset(() => ({ path: { asset_identity: props.asset_identity } })) : undefined;
    const asset = createMemo(() => ("asset" in props ? props.asset : assetQuery?.data));

    return (
        <AssetIconImage asset={asset()} />
    );
};
