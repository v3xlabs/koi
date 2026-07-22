import { createFileRoute } from "@tanstack/solid-router";
import { For, Show, Suspense } from "solid-js";

import { useNetworks } from "#/api/network";
import { NetworkAdd } from "#/components/net/add";
import { NetworkEdit } from "#/components/net/edit";

export const Route = createFileRoute("/settings/networks")({
  component: () => {
    const networksQuery = useNetworks();

    return (
      <div class="w-full space-y-4">
        <div class="flex justify-between items-center w-full">
          <div class="">
            <div class="text-xl font-bold">
              Networks
            </div>
            <div class="text-sm text-muted">
              These are the networks you have added system-wide.
            </div>
          </div>
          <div>
            <NetworkAdd />
          </div>
        </div>
        <Suspense fallback={<div class="py-8 text-center text-muted">Loading...</div>}>
          <Show when={networksQuery.data}>
            {data => (
              <div class="space-y-4">
                <For each={data().networks}>
                  {network => (
                    <NetworkEdit network_identity={network.network_identity} />
                  )}
                </For>
              </div>
            )}
          </Show>
        </Suspense>
      </div>
    );
  },
});
