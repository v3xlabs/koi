import { createFileRoute, useParams } from "@tanstack/solid-router";

import { AccountTxHistory } from "#/components/account/tx/list";

export const Route = createFileRoute("/acc/$account/history")({
  component: () => {
    const params = useParams({ from: "/acc/$account/history" });
    const account_identity = Number.parseInt(params().account);

    return (
      <div class="w-full p-4">
        <div class="w-full max-w-4xl space-y-4">
          <div class="flex items-end justify-between">
            <div class="text-xl">
              History
            </div>
          </div>
        </div>
        <AccountTxHistory account_identity={account_identity} />
      </div>
    );
  },
});
