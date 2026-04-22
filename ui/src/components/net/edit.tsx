import { Component, For, Show, Suspense } from "solid-js";

import { useNetwork, useNetworkEndpoints, useUpdateNetwork } from "#/api/network";

import { NetworkDelete } from "./delete";
import { NetworkEndpointAdd } from "./endpoint/add";
import { NetworkEndpointItem } from "./endpoint/edit";
import { NetworkEndpointPreview } from "./endpoint/preview";
import { NetworkEndpointCollapsible } from "./endpoint/collapsible";

const NetworkEndpoints: Component<{ network_id: number; }> = ({ network_id }) => {
    const networkEndpointsQuery = useNetworkEndpoints(() => ({
        path: {
            network_id,
        },
    }));

    return (
        <div class="space-y-2">
            <div class="flex justify-between items-end">
                <div>Endpoints</div>
                <NetworkEndpointAdd network_id={network_id} />
            </div>
            <ul class="border border-border rounded-md">
                <Suspense fallback={<div>Loading...</div>}>
                    <Show when={networkEndpointsQuery.data}>
                        {data => (
                            <For each={data()}>
                                {endpoint => (
                                    <NetworkEndpointCollapsible network_id={network_id} endpoint_id={endpoint.endpoint_identity} />
                                )}
                            </For>
                        )}
                    </Show>
                </Suspense>
            </ul>
        </div>
    );
};

export const NetworkEdit: Component<{ network_id: number; }> = ({ network_id }) => {
    const networkQuery = useNetwork(() => ({
        path: {
            network_id,
        },
    }));

    // TODO: Implement name editing (and icon)
    const updateNetwork = useUpdateNetwork(() => ({
        path: {
            network_id,
        },
    }));

    return (
        <div>
            <div class="flex justify-between items-center">
                <div>
                    <div>{networkQuery.data?.network_name}</div>
                    <div>
                        #
                        {networkQuery.data?.network_identity?.toString()}
                    </div>
                </div>

                <NetworkDelete network_id={network_id} />
            </div>
            <NetworkEndpoints network_id={network_id} />
        </div>
    );
};
