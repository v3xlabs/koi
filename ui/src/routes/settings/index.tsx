import { createFileRoute } from "@tanstack/solid-router";

export const Route = createFileRoute("/settings/")({
  component: () => (
      <div class="p-4 mx-auto w-full max-w-lg">
        <div class="text-lg">
          Settings
        </div>
        <div class="bg-surface p-4 rounded-md w-full">
          <div>
          </div>
        </div>
      </div>
    ),
});
