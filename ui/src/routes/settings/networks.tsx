import { createFileRoute } from "@tanstack/solid-router";
import { For, Suspense } from "solid-js";

import { useNetworks } from "#/api/network";

export const Route = createFileRoute("/settings/networks")({
  component: () => {
    const networksQuery = useNetworks();

    return (
      <div class="w-full">
        <div class="text-lg">
          Networks
        </div>
        <div class="bg-surface p-4 rounded-md w-full">
          <Suspense fallback={<div>Loading...</div>}>
            <For each={networksQuery.data?.networks}>
              {network => (
                <div>{network.network_name}</div>
              )}
            </For>
          </Suspense>
        </div>
      </div>
    );
  },
});
