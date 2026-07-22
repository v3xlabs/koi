import { Collapsible } from "@kobalte/core/collapsible";
import { FaSolidChevronDown } from "solid-icons/fa";
import { Component, createMemo, Show, Suspense } from "solid-js";

import { useNetworkEndpoint } from "#/api/network";

import { NetworkEndpointItem } from "./edit";
import { NetworkEndpointPreview } from "./preview";

export const NetworkEndpointCollapsible: Component<{ network_identity: number; endpoint_identity: number; }> = ({ network_identity, endpoint_identity }) => {
    const endpointQuery = useNetworkEndpoint(() => ({
        path: {
            network_identity,
            endpoint_identity,
        },
    }));

    const endpoint = createMemo(() => endpointQuery.data);

    return (
        <Collapsible class="border-t border-border first:border-t-0">
            <Collapsible.Trigger class="w-full flex items-center justify-between pr-4 group cursor-pointer hover:bg-surface-alt">
                <NetworkEndpointPreview network_identity={network_identity} endpoint_identity={endpoint_identity} />
                <FaSolidChevronDown class="text-muted group-hover:text-foreground" />
            </Collapsible.Trigger>
            <Collapsible.Content class="border-t border-border">
                <Suspense>
                    <Show when={endpoint()}>
                        {endpoint => (
                            <NetworkEndpointItem network_identity={network_identity} endpoint={endpoint()} />
                        )}
                    </Show>
                </Suspense>
            </Collapsible.Content>
        </Collapsible>
    );
};
