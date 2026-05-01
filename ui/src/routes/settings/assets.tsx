import { createFileRoute } from "@tanstack/solid-router";

import { AssetList } from "#/components/asset/list";

export const Route = createFileRoute("/settings/assets")({
  component: () => (
      <div class="w-full space-y-2">
        <div class="flex justify-between items-end">
          <div class="text-lg">
            Assets
          </div>
        </div>
        <AssetList />
      </div>
    ),
});
