import { Tooltip } from "@kobalte/core/tooltip";
import { Component, Show, Suspense } from "solid-js";

import { useNetworkEndpoint, useNetworkEndpointStatus } from "#/api/network";
import { narrow } from "#/utils/narrow";

import { NetworkEndpointStatus } from "./status";

export const NetworkEndpointPreview: Component<{ network_identity: number; endpoint_identity: number; }> = ({ network_identity, endpoint_identity }) => {
    const endpointQuery = useNetworkEndpoint(() => ({
        path: {
            network_identity,
            endpoint_identity,
        },
    }));
    const statusQuery = useNetworkEndpointStatus(() => ({
        path: {
            network_identity,
            endpoint_identity,
        },
    }));

    return (
        <Suspense>
            <div class="grid min-w-0 flex-1 grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-3 px-4 py-3 text-left">
                    <div class="flex items-center">
                        <Suspense>
                            <Show when={statusQuery.data}>
                                {data => (
                                    <Tooltip>
                                        <Tooltip.Trigger>
                                            <NetworkEndpointStatus status={() => data().status} />
                                        </Tooltip.Trigger>
                                        <Tooltip.Portal>
                                            <Tooltip.Content class="bg-surface-alt text-secondary-foreground rounded-md border border-border p-2 text-xs">
                                                <Tooltip.Arrow />
                                                {data().status}
                                            </Tooltip.Content>
                                        </Tooltip.Portal>
                                    </Tooltip>
                                )}
                            </Show>
                        </Suspense>
                    </div>
                    <div class="min-w-0">
                        <div class="flex items-center gap-2">
                            <span class="truncate font-medium">{endpointQuery.data?.endpoint_label || "Unnamed endpoint"}</span>
                            <span class="text-xs text-muted uppercase">{endpointQuery.data?.endpoint_type}</span>
                        </div>
                        <div class="truncate text-xs text-muted">{endpointQuery.data?.endpoint_url}</div>
                    </div>
                    <div class="text-right text-xs tabular-nums text-muted">
                        <Suspense>
                            <Show when={statusQuery.data}>
                                {data => (
                                    <Show when={narrow(() => data(), value => value.status === "Alive")}>
                                        {alive => (
                                            <span>
                                                Block
                                                {" "}
                                                {alive().block_number}
                                            </span>
                                        )}
                                    </Show>
                                )}
                            </Show>
                        </Suspense>
                    </div>
            </div>
        </Suspense>
    );
};
