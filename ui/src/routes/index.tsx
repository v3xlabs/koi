import { createFileRoute, Link } from "@tanstack/solid-router";
import { FiPlus } from "solid-icons/fi";
import { For } from "solid-js";

import { useAccounts } from "#/api/account";
import { AccountPreview } from "#/components/account/preview";

export const Route = createFileRoute("/")({
  component: () => {
    const accounts = useAccounts();

    return (
      <div class="w-full p-4">
        <div class="mx-auto w-full max-w-lg space-y-4 mt-4">
          <div class="flex items-end justify-between">
            <div class="text-xl">
              Accounts
            </div>
            <div>
              <button class="btn btn-primary flex items-center gap-2 text-sm">
                <FiPlus />
                Add
              </button>
            </div>
          </div>
          <div class="bg-surface py-4 rounded-md w-full">
            <div class="space-y-2">
              <For each={accounts}>
                {account => (
                  <Link to="/acc/$account" params={{ account: account.account_id.toString() }} class="w-full px-2 py-1 bg-surface-alt text-sm font-bold flex">
                    <AccountPreview account_id={account.account_id} />
                  </Link>
                )}
              </For>
            </div>
          </div>
        </div>
      </div>
    );
  },
});
