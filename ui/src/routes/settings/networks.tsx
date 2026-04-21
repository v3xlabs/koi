import { createFileRoute } from "@tanstack/solid-router";
import { For, Show, Suspense } from "solid-js";

import { useNetworks } from "#/api/network";
import { NetworkAdd } from "#/components/net/add";

export const Route = createFileRoute("/settings/networks")({
  component: () => {
    const networksQuery = useNetworks();

    return (
      <div class="w-full">
        <div class="flex justify-between items-center">
          <div class="text-lg">
            Networks
          </div>
          <div>
            <NetworkAdd />
          </div>
        </div>
        <div class="bg-surface p-4 rounded-md w-full">
          <Suspense fallback={<div>Loading...</div>}>
            <Show when={networksQuery.data}>
              {data => (
                <For each={data().networks}>
                  {network => (
                    <div>{network.network_name}</div>
                  )}
                </For>
              )}
            </Show>
          </Suspense>
        </div>
      </div>
    );
  },
});
