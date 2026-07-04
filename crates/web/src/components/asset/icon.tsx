import { Component, createMemo, Show } from "solid-js";

import { Asset, useAsset } from "#/api/asset";
import { cacheImageUrl } from "#/utils/image-cache";

type AssetIconProps = { asset: Asset; class?: string; } | { asset_identity: string; class?: string; };

const textToColor = (text: string) => `#${text.split("").reduce((hash, char) => char.charCodeAt(0) + ((hash << 5) - hash), 0)
    .toString(16)
    .padStart(6, "0")}`;

export const AssetIconImage: Component<{ asset?: Asset; class?: string; }> = props => (
    <Show
      when={props.asset?.asset_icon_url}
      fallback={(
            <div
              classList={{
                    [props.class ?? "size-6"]: true,
                    "aspect-square rounded-full": true,
                }}
              style={{
                    "background-color": textToColor(props.asset?.asset_identity ?? ""),
                }}
            />
        )}
    >
        {icon => (
            <img
              src={cacheImageUrl(icon())}
              alt={props.asset?.asset_name}
              classList={{
                    [props.class ?? "size-6"]: true,
                    "aspect-square rounded-full": true,
                }}
            />
        )}
    </Show>
);

export const AssetIcon: Component<AssetIconProps> = (props) => {
    const assetQuery = "asset_identity" in props ? useAsset(() => ({ path: { asset_identity: props.asset_identity } })) : undefined;
    const asset = createMemo(() => ("asset" in props ? props.asset : assetQuery?.data));

    return (
        <AssetIconImage asset={asset()} class={props.class} />
    );
};
