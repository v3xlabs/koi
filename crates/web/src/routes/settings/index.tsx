import { createFileRoute } from "@tanstack/solid-router";

import { DisplayCurrencySelector } from "#/components/quoter/display";

export const Route = createFileRoute("/settings/")({
  component: () => (
    <div class="w-full">
      <div class="text-lg">
        Settings
      </div>
      <div class="bg-surface p-4 rounded-md w-full">
        <DisplayCurrencySelector showLabel />
      </div>
    </div>
  ),
});
