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
        <div class="bg-surface p-4 rounded-md w-full divide-y divide-border">
          <Suspense fallback={<div>Loading...</div>}>
            <Show when={networksQuery.data}>
              {data => (
                <For each={data().networks}>
                  {network => (
                    <NetworkEdit network_id={network.network_identity} />
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
