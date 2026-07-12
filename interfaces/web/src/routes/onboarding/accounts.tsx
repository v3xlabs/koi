import { createFileRoute } from "@tanstack/solid-router";

import { AccountEdit } from "#/components/account/edit";

export const Route = createFileRoute("/onboarding/accounts")({
  component: () => (
      <div class="w-full p-4 bg-surface rounded-md border border-border">
        <AccountEdit />
      </div>
    ),
});
