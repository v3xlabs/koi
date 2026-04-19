import { createFileRoute } from "@tanstack/solid-router";

import { AddressInput } from "#/components/input/address";

export const Route = createFileRoute("/acc/import/view")({
  component: () => (
    <div class="p-4 mx-auto w-full max-w-lg">
      <div>
        Import View
      </div>
      <div class="bg-surface p-4 rounded-md w-full space-y-4">
        <div class="space-y-4">
          <label class="space-y-1 block">
            <span class="block">Address</span>
            <AddressInput placeholder="0x123...456" class="w-full" />
          </label>
          <label class="space-y-1 block">
            <span class="block">Name</span>
            <input type="text" class="input w-full" />
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
