import { createFileRoute } from "@tanstack/solid-router";

export const Route = createFileRoute("/settings/networks")({
  component: () => (
    <div class="w-full">
      <div class="text-lg">
        Networks
      </div>
      <div class="bg-surface p-4 rounded-md w-full">

      </div>
    </div>
  ),
});
