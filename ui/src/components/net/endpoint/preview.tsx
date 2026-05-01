import { FaSolidCubes } from "solid-icons/fa";
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
            <div class="flex justify-between items-center w-full p-4">
                <div class="flex items-center gap-2">
                    <Suspense>
                        <Show when={statusQuery.data}>
                            {data => (
                                <NetworkEndpointStatus status={() => data().status} />
                            )}
                        </Show>
                    </Suspense>
                    <span>{endpointQuery.data?.endpoint_label}</span>
                </div>
                <div>
                    <Suspense>
                        <Show when={statusQuery.data}>
                            {data => (
                                <div class="text-xs text-start">
                                    <span>{data().status}</span>
                                    <Show when={narrow(() => data(), x => x.status === "Alive")}>
                                        {data => (
                                            <span class="flex items-center gap-1">
                                                <FaSolidCubes />
                                                <span>
                                                    {data().block_number}
                                                </span>
                                            </span>
                                        )}
                                    </Show>
                                </div>
                            )}
                        </Show>
                    </Suspense>
                </div>
            </div>
        </Suspense>
    );
};
