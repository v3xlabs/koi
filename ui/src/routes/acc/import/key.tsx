import { createFileRoute } from "@tanstack/solid-router";

export const Route = createFileRoute("/acc/import/key")({
  component: () => (
    <div class="p-4 mx-auto w-full max-w-lg">
      <div>
        Import Key
      </div>
      <div class="bg-surface p-4 rounded-md w-full space-y-4">
        <div class="space-y-4">
          <label class="space-y-1 block">
            <span class="block">Private Key</span>
            <textarea class="input w-full" rows={4} />
          </label>
        </div>
        <div class="flex justify-end">
          <button class="btn btn-primary" disabled>
            Import
          </button>
        </div>
      </div>
    </div>
  ),
});
