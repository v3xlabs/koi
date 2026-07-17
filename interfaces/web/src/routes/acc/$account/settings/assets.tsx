import { createFileRoute, useParams } from "@tanstack/solid-router";
import { For, Show, Suspense } from "solid-js";

import { useAccountAssets } from "#/api/account";
import { AccountAssetLink, AssetUnlink } from "#/components/account/asset/link";
import { AssetPreview } from "#/components/asset/preview";

export const Route = createFileRoute("/acc/$account/settings/assets")({
  component: () => {
    const params = useParams({ from: "/acc/$account" });
    const account_identity = Number.parseInt(params().account);

    const assetsQuery = useAccountAssets(() => ({ path: { account_identity } }));

    return (
      <div class="bg-surface p-4 rounded-md w-full space-y-4">
        <div>
          <Suspense fallback={<div>Loading...</div>}>
            <Show when={assetsQuery.data && assetsQuery.data.length > 0} fallback={<div>No assets enabled to this account</div>}>
              <For each={assetsQuery.data}>
                {asset => (
                  <div class="flex items-center gap-2">
                    <AssetPreview asset_identity={asset} />
                    <AssetUnlink account_identity={account_identity} asset_identity={asset} />
                  </div>
                )}
              </For>
            </Show>
          </Suspense>
        </div>
        <AccountAssetLink account_identity={account_identity} />
      </div>
    );
  },
});
