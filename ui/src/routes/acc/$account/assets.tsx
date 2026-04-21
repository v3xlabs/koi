import { createFileRoute, useParams } from "@tanstack/solid-router";

import { AccountAssetTable } from "#/components/account/asset/table";

export const Route = createFileRoute("/acc/$account/assets")({
  component: () => {
    const params = useParams({ from: "/acc/$account/assets" });

    return (
      <div class="w-full p-4">
        <div class="w-full max-w-4xl space-y-4">
          <div class="flex items-end justify-between">
            <div class="text-xl">
              Assets
            </div>
          </div>
          <AccountAssetTable account_id={params().account} />
        </div>
      </div>
    );
  },
});
