import { createFileRoute } from "@tanstack/solid-router";

import { DisplayCurrencySelector } from "#/components/quoter/display";

export const Route = createFileRoute("/settings/")({
  component: () => (
    <div class="w-full space-y-4">
      <div class="">
        <div class="text-xl font-bold">
          Settings
        </div>
        <div class="text-sm text-muted">
          These are system-wide general settings.
        </div>
      </div>
      <div class="bg-surface rounded-md p-4">
        <DisplayCurrencySelector showLabel />
      </div>
    </div>
  ),
});
