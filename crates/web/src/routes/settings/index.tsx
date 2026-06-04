import { createFileRoute } from "@tanstack/solid-router";

import { DisplayCurrencySelector } from "#/components/quoter/display";

export const Route = createFileRoute("/settings/")({
  component: () => (
    <div class="w-full space-y-4">
      <div class="text-xl font-bold">
        Settings
      </div>
      <DisplayCurrencySelector showLabel />
    </div>
  ),
});
