import { createFileRoute } from "@tanstack/solid-router";

export const Route = createFileRoute("/onboarding/vendors")({
  component: () => (
    <div class="w-full p-4 bg-surface rounded-md border border-border">
      <div>
        Vendors
      </div>
    </div>
  ),
});
