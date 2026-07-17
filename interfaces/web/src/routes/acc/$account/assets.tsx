import { createFileRoute, useParams } from "@tanstack/solid-router";

import { AccountAssetTable } from "#/components/account/asset/table";

export const Route = createFileRoute("/acc/$account/assets")({
  component: () => {
    const params = useParams({ from: "/acc/$account/assets" });
    const account_id = Number.parseInt(params().account);

    return (
      <div class="w-full pb-64">
        <AccountAssetTable account_identity={account_id} />
      </div>
    );
  },
});
