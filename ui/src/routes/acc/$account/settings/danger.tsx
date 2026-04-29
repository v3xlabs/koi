import { createFileRoute } from "@tanstack/solid-router";

export const Route = createFileRoute("/acc/$account/settings/danger")({
  component: () => (
    <div class="px-4">
      <div class="bg-surface p-4 rounded-md w-full space-y-4">
        <div>
          Danger
        </div>
        <div>
          <button class="btn btn-danger" disabled>
            Remove Account
          </button>
        </div>
      </div>
    </div>
  ),
});

