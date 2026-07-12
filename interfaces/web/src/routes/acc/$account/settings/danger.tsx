import { createFileRoute, useNavigate, useParams } from "@tanstack/solid-router";

import { useAccount } from "#/api/account";
import { AccountDelete } from "#/components/account/delete";

export const Route = createFileRoute("/acc/$account/settings/danger")({
  component: () => {
    const navigate = useNavigate();
    const params = useParams({ from: "/acc/$account" });
    const accountQuery = useAccount(() => ({ path: { account_identity: Number.parseInt(params().account) } }));

    return (
        <div class="bg-surface p-4 rounded-md w-full space-y-4">
          <div>
            Danger
          </div>
          <div>
            <AccountDelete
              account_identity={Number.parseInt(params().account)}
              account_name={accountQuery.data?.name ?? "Account"}
              onDeleted={() => navigate({ to: "/" })}
            />
          </div>
        </div>
    );
  },
});
