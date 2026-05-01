import { Component, For, Show } from "solid-js";

import { useAssets } from "#/api/asset";

import { AssetAdd } from "./add";

export const AssetList: Component = () => {
    const assetsQuery = useAssets();

    return (
        <div class="w-full space-y-2 bg-surface rounded-md p-4">
            <div>
                <AssetAdd />
            </div>
            <Show when={assetsQuery.data?.assets?.length && assetsQuery.data?.assets?.length > 0} fallback={<div class="text-center text-muted">No assets found</div>}>
                <ul>
                    <For each={assetsQuery.data?.assets} fallback={<div class="text-center text-muted">No assets found</div>}>
                        {asset => (
                            <div>{asset.asset_name}</div>
                        )}
                    </For>
                </ul>
            </Show>
        </div>
    );
};
