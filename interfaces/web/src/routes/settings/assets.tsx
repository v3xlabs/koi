import { createFileRoute } from "@tanstack/solid-router";
import { createMemo } from "solid-js";
import { For, Show } from "solid-js/web";

import { useAssets } from "#/api/asset";
import { AssetAdd } from "#/components/asset/add";
import { AssetPreview } from "#/components/asset/preview";
import { AssetQuote } from "#/components/asset/quote";

export const Route = createFileRoute("/settings/assets")({
  component: () => {
    const assetsQuery = useAssets();

    const assets = createMemo(() => (assetsQuery.data?.assets ?? []).toSorted((a, b) => (
      b.asset_identity.localeCompare(a.asset_identity)
    )));

    return (
      <div class="w-full space-y-4">
        <div class="flex justify-between items-center">
          <div class="">
            <div class="text-xl font-bold">
              Assets
            </div>
            <div class="text-sm text-muted">
              These are the tokens and currencies you have added system-wide.
            </div>
          </div>
          <AssetAdd />
        </div>
        <div class="bg-surface rounded-md p-4">
          <Show when={assets().length > 0} fallback={<div class="py-8 text-center text-muted">No assets found</div>}>
            <ul class="space-y-1">
              <For each={assets()}>
                {asset => (
                  <li class="py-2 px-2 hover:bg-surface-alt rounded-lg flex items-center justify-between gap-2">
                    <AssetPreview asset_identity={asset.asset_identity} />
                    <AssetQuote asset_identity={asset.asset_identity} />
                  </li>
                )}
              </For>
            </ul>
          </Show>
        </div>
      </div>
    );
  },
});
