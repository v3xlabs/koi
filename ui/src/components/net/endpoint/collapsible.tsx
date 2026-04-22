import { Collapsible } from "@kobalte/core/collapsible";
import { Component, createMemo, Show, Suspense } from "solid-js";

import { useNetworkEndpoint } from "#/api/network";

import { NetworkEndpointItem } from "./edit";
import { NetworkEndpointPreview } from "./preview";
import { FaSolidChevronDown } from "solid-icons/fa";

export const NetworkEndpointCollapsible: Component<{ network_id: number; endpoint_id: string; }> = ({ network_id, endpoint_id }) => {
    const endpointQuery = useNetworkEndpoint(() => ({
        path: {
            network_id,
            endpoint_id,
        },
    }));

    const endpoint = createMemo(() => endpointQuery.data);

    return (
        <Collapsible>
            <Collapsible.Trigger class="w-full flex items-center justify-between pr-4 group cursor-pointer hover:bg-surface-alt">
                <NetworkEndpointPreview network_id={network_id} endpoint_id={endpoint_id} />
                <FaSolidChevronDown class="text-muted group-hover:text-foreground" />
            </Collapsible.Trigger>
            <Collapsible.Content class="border-t border-border">
                <Suspense>
                    <Show when={endpoint()}>
                        {endpoint => (
                            <NetworkEndpointItem network_id={network_id} endpoint={endpoint()} />
                        )}
                    </Show>
                </Suspense>
            </Collapsible.Content>
        </Collapsible>
    );
};
