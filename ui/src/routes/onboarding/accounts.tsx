import { createFileRoute } from "@tanstack/solid-router";

export const Route = createFileRoute("/onboarding/accounts")({
  component: () => (
    <div class="w-full p-4 bg-surface rounded-md border border-border">
      <div>
        Accounts
      </div>
    </div>
  ),
});
