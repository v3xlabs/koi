import { createFileRoute } from "@tanstack/solid-router";

import { VendorEdit } from "#/components/vendor/edit";

export const Route = createFileRoute("/settings/vendors")({
  component: () => (
    <div class="space-y-4">
      <div class="">
        <div class="text-xl font-bold">
          Vendors
        </div>
        <div class="text-sm text-muted">
          All non-rpc calls are gated behind vendor flags.
        </div>
      </div>
      <div class="bg-surface rounded-md p-4">
        <VendorEdit />
      </div>
    </div>
  ),
});
