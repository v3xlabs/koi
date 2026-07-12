import { createFileRoute, useParams } from "@tanstack/solid-router";

import { AccountTxHistory } from "#/components/account/tx/list";

export const Route = createFileRoute("/acc/$account/history")({
  component: () => {
    const params = useParams({ from: "/acc/$account/history" });
    const account_identity = Number.parseInt(params().account);

    return (
      <div class="w-full">
        <AccountTxHistory account_identity={account_identity} />
      </div>
    );
  },
});
