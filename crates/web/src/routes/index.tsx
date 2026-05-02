import { createFileRoute, Link } from "@tanstack/solid-router";
import { FiMoreVertical, FiPlus } from "solid-icons/fi";
import { For, Suspense } from "solid-js";

import { useAccounts } from "#/api/account";
import { AccountPreview } from "#/components/account/preview";
import { button } from "#/components/input/button";

export const Route = createFileRoute("/")({
  component: () => {
    const accountsQuery = useAccounts();

    return (
      <div class="w-full p-4">
        <div class="mx-auto w-full max-w-2xl space-y-6 mt-8">
          <div class="flex items-end justify-between">
            <div class="text-3xl font-bold">
              Accounts
            </div>
            <div>
              <Link to="/acc/new" class={button({ variant: "primary", class: "text-sm" })}>
                <FiPlus />
                Add
              </Link>
            </div>
          </div>
          <div class="bg-surface py-5 px-4 rounded-xl w-full">
            <div class="space-y-1">
              <Suspense fallback={<div class="py-8 text-center text-muted">Loading...</div>}>
                <For each={accountsQuery.data?.accounts}>
                  {account => (
                    <Link to="/acc/$account" params={{ account: account.account_identity.toString() }} class="w-full px-2 hover:bg-surface-alt rounded-lg py-2 text-sm font-bold flex items-center gap-2">
                      <AccountPreview account_identity={account.account_identity} />
                      <button class={button({ variant: "ghost", size: "small", square: true })}>
                        <FiMoreVertical />
                      </button>
                    </Link>
                  )}
                </For>
              </Suspense>
            </div>
          </div>
        </div>
      </div>
    );
  },
});
