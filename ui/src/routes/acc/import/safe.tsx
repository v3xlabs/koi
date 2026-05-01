import { createFileRoute } from "@tanstack/solid-router";

export const Route = createFileRoute("/acc/import/safe")({
  component: () => (
    <div class="p-4 mx-auto w-full max-w-lg">
      <div>
        Import Safe
      </div>
    </div>
  ),
});
