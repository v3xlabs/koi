import { createFileRoute } from "@tanstack/solid-router";

export const Route = createFileRoute("/acc/$account/new-tx")({
  component: () => (
    <div class="p-4">
      <div class="bg-surface p-4 rounded-md w-full">
        <div>
          <div>
            New transaction
          </div>
        </div>
      </div>
    </div>
  ),
});
