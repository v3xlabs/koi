import { Component, Show, Suspense } from "solid-js";

import { useNetworkEndpoint, useNetworkEndpointStatus } from "#/api/network";

import { NetworkEndpointStatus } from "./status";
import { narrow } from "#/utils/narrow";
import { FaSolidCube, FaSolidCubes } from "solid-icons/fa";

export const NetworkEndpointPreview: Component<{ network_id: number; endpoint_id: string; }> = ({ network_id, endpoint_id }) => {
    const endpointQuery = useNetworkEndpoint(() => ({
        path: {
            network_id,
            endpoint_id,
        },
    }));
    const statusQuery = useNetworkEndpointStatus(() => ({
        path: {
            network_id,
            endpoint_id,
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
