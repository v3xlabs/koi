import { createFileRoute } from "@tanstack/solid-router";
import { createMemo } from "solid-js";
import { For, Show } from "solid-js/web";

import { useAssets } from "#/api/asset";
import { AssetAdd } from "#/components/asset/add";
import { AssetPreview } from "#/components/asset/preview";

export const Route = createFileRoute("/settings/assets")({
  component: () => {
    const assetsQuery = useAssets();

    const assets = createMemo(() => (assetsQuery.data?.assets ?? []).toSorted((a, b) => (
      b.asset_identity.localeCompare(a.asset_identity)
    )));

    return (
      <div class="w-full space-y-2">
        <div class="flex justify-between items-end">
          <div class="text-lg">
            Assets
          </div>
          <div>
            <AssetAdd />
          </div>
        </div>
        <div class="w-full space-y-2 bg-surface rounded-md p-4">
          <Show when={assets().length > 0} fallback={<div class="text-center text-muted">No assets found</div>}>
            <ul>
              <For each={assets()}>
                {asset => (
                  <div class="py-2 px-4 hover:bg-surface-alt cursor-pointer rounded-md flex items-center justify-between">
                    <AssetPreview asset_identity={asset.asset_identity} />
                  </div>
                )}
              </For>
            </ul>
          </Show>
        </div>
      </div>
    );
  },
});
