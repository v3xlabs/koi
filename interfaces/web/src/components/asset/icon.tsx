import { Component, createEffect, createMemo, createSignal, onCleanup, Show } from "solid-js";

import { Asset, useAsset, useAssetIcon } from "#/api/asset";

type AssetIconProps = { asset: Asset; class?: string; } | { asset_identity: string; class?: string; };

const textToColor = (text: string) => `#${Array.from(text).reduce((hash, char) => (char.codePointAt(0) ?? 0) + ((hash << 5) - hash), 0)
    .toString(16)
    .padStart(6, "0")}`;

export const AssetIconImage: Component<{ asset?: Asset; iconUrl?: string; class?: string; }> = props => (
    <Show
      when={props.iconUrl}
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
              src={icon()}
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
    const assetIdentity = createMemo(() => ("asset" in props ? props.asset.asset_identity : props.asset_identity));
    const iconQuery = useAssetIcon(() => ({ path: { asset_identity: assetIdentity() } }));
    const [iconUrl, setIconUrl] = createSignal<string>();

    createEffect(() => {
        const icon = iconQuery.data;

        if (!icon) {
            setIconUrl();

            return;
        }

        const url = URL.createObjectURL(new Blob([new Uint8Array(icon.png_data)], { type: "image/png" }));

        setIconUrl(url);
        onCleanup(() => URL.revokeObjectURL(url));
    });

    return (
        <AssetIconImage asset={asset()} iconUrl={iconUrl()} class={props.class} />
    );
};
