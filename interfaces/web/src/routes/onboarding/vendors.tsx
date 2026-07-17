import { createFileRoute } from "@tanstack/solid-router";

import { VendorEdit } from "#/components/vendor/edit";

export const Route = createFileRoute("/onboarding/vendors")({
  component: VendorEdit,
});
