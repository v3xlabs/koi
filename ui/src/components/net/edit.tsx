import { Tabs } from "@kobalte/core/tabs";
import { Component, For, Show, Suspense } from "solid-js";

import { useNetwork, useNetworkEndpoints, useUpdateNetwork } from "#/api/network";

import { NetworkDelete } from "./delete";
import { NetworkEndpointAdd } from "./endpoint/add";
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
                    <Show when={networkEndpointsQuery.data?.length === 0}>
                        <div class="p-4 text-muted text-center">
                            No endpoints found
                        </div>
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
            <Tabs>
                <div class="flex justify-between items-end">
                    <div class="flex items-center gap-2 px-1 pb-2 pt-1">
                        <Show when={networkQuery.data?.network_icon_url}>
                            {icon => <img src={icon()} alt={networkQuery.data?.network_name} class="size-4 aspect-square rounded-full" />}
                        </Show>
                        <div class="flex items-baseline gap-1">
                            <span>{networkQuery.data?.network_name}</span>
                            <span class="text-muted text-xs">
                                #
                                {networkQuery.data?.network_identity?.toString()}
                            </span>
                        </div>
                    </div>

                    <div class="flex justify-end">
                        <Tabs.List class="flex gap-1">
                            <For each={[{ value: "endpoints", label: "Endpoints" }, { value: "details", label: "Details" }]}>
                                {item => (
                                    <Tabs.Trigger value={item.value} class="btn btn-secondary border-b-0 !rounded-b-none border-x border-t border-border btn-small text-sm">{item.label}</Tabs.Trigger>
                                )}
                            </For>
                        </Tabs.List>
                    </div>
                </div>
                <div class="bg-surface p-4 rounded-md w-full">
                    <Tabs.Content value="endpoints">
                        <NetworkEndpoints network_id={network_id} />
                    </Tabs.Content>
                    <Tabs.Content value="details">
                        <div class="w-full space-y-2">
                            <div class="flex gap-2 w-full">
                                <div class="grow">
                                    <div>Name</div>
                                    <input type="text" class="input w-full" value={networkQuery.data?.network_name} disabled />
                                </div>
                                <div>
                                    <div>Network Id</div>
                                    <input type="text" class="input w-full" value={networkQuery.data?.network_identity?.toString()} disabled />
                                </div>
                            </div>

                            <div class="flex gap-2 w-full items-center">
                                <Show when={networkQuery.data?.network_icon_url}>
                                    {icon => <img src={icon()} alt={networkQuery.data?.network_name} class="size-6 aspect-square rounded-full mx-2" />}
                                </Show>
                                <div class="grow">
                                    <div>Icon</div>
                                    <input type="text" class="input w-full" value={networkQuery.data?.network_icon_url} disabled />
                                </div>
                            </div>

                            <div class="flex w-full justify-end">
                                <NetworkDelete network_id={network_id} />
                            </div>
                        </div>
                    </Tabs.Content>
                </div>
            </Tabs>
        </div>
    );
};
