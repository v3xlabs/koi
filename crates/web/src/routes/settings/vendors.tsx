import { createFileRoute } from "@tanstack/solid-router";

import { VendorEdit } from "#/components/vendor/edit";

export const Route = createFileRoute("/settings/vendors")({
  component: () => (
    <div>
      <VendorEdit />
    </div>
  ),
});
