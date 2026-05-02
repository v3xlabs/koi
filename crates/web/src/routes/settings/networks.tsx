import { createFileRoute } from "@tanstack/solid-router";
import { For, Show, Suspense } from "solid-js";

import { useNetworks } from "#/api/network";
import { NetworkAdd } from "#/components/net/add";
import { NetworkEdit } from "#/components/net/edit";

export const Route = createFileRoute("/settings/networks")({
  component: () => {
    const networksQuery = useNetworks();

    return (
      <div class="w-full space-y-2">
        <div class="flex justify-between items-end">
          <div class="text-lg">
            Networks
          </div>
          <div>
            <NetworkAdd />
          </div>
        </div>
        <Suspense fallback={<div>Loading...</div>}>
          <Show when={networksQuery.data}>
            {data => (
              <For each={data().networks}>
                {network => (
                  <NetworkEdit network_identity={network.network_identity} />
                )}
              </For>
            )}
          </Show>
        </Suspense>
      </div>
    );
  },
});
