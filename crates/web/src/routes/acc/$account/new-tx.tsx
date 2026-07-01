import { createFileRoute } from "@tanstack/solid-router";

import { TxBuilder } from "#/components/tx/builder/builder";

export const Route = createFileRoute("/acc/$account/new-tx")({
  component: () => (
    <div class="w-full space-y-4">
      <div class="">
        <div class="text-lg font-bold">New transaction</div>
        <div class="text-sm text-muted">
          Build transactions step by step
        </div>
      </div>
      <TxBuilder />
    </div>
  ),
});
