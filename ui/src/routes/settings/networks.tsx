import { createFileRoute } from "@tanstack/solid-router";
import { Component, For, Show, Suspense } from "solid-js";

import { useNetworkEndpoints, useNetworks } from "#/api/network";
import { NetworkAdd } from "#/components/net/add";
import { NetworkDelete } from "#/components/net/delete";

const NetworkEndpoints: Component<{ network_id: number; }> = ({ network_id }) => {
  const networkEndpointsQuery = useNetworkEndpoints(() => ({
    path: {
      network_id,
    },
  }));

  return (
    <div>
      <div>Network Endpoints</div>
      <ul>
        <Suspense fallback={<div>Loading...</div>}>
          <Show when={networkEndpointsQuery.data}>
            {data => (
              <For each={data()}>
                {endpoint => (
                  <li>
                    {endpoint.endpoint_label}
                  </li>
                )}
              </For>
            )}
          </Show>
        </Suspense>
      </ul>
    </div>
  );
};

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
                    <div>
                      <div class="flex justify-between items-center">
                        <div>
                          <div>{network.network_name}</div>
                          <div>
                            #
                            {network.network_identity.toString()}
                          </div>
                        </div>

                        <NetworkDelete network_id={network.network_identity} />
                      </div>
                      <NetworkEndpoints network_id={network.network_identity} />
                    </div>
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
