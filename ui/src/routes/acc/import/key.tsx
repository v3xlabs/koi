import { createFileRoute } from "@tanstack/solid-router";

export const Route = createFileRoute("/acc/import/key")({
  component: () => (
    <div class="p-4">
      <div class="bg-surface p-4 rounded-md w-full">
        <div>
          <div>
            Import Key
          </div>
        </div>
      </div>
    </div>
  ),
});
