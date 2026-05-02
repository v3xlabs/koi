import { Tabs } from "@kobalte/core/tabs";
import { Component, createMemo, For, Show, Suspense } from "solid-js";

import { useNetwork, useNetworkEndpoints } from "#/api/network";
import { button } from "#/components/input/button";

import { NetworkDelete } from "./delete";
import { NetworkEndpointAdd } from "./endpoint/add";
import { NetworkEndpointCollapsible } from "./endpoint/collapsible";

const NetworkEndpoints: Component<{ network_identity: number; }> = ({ network_identity }) => {
    const networkEndpointsQuery = useNetworkEndpoints(() => ({
        path: {
            network_identity,
        },
    }));

    return (
        <div class="space-y-2">
            <div class="flex justify-between items-end">
                <div>Endpoints</div>
                <NetworkEndpointAdd network_identity={network_identity} />
            </div>
            <ul class="border border-border rounded-md">
                <Suspense fallback={<div>Loading...</div>}>
                    <Show when={networkEndpointsQuery.data}>
                        {data => (
                            <For each={data()}>
                                {endpoint => (
                                    <NetworkEndpointCollapsible network_identity={network_identity} endpoint_identity={endpoint.endpoint_identity} />
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

export const NetworkEdit: Component<{ network_identity: number; }> = ({ network_identity }) => {
    const networkQuery = useNetwork(() => ({
        path: {
            network_identity,
        },
    }));
    const network = createMemo(() => networkQuery.data);
    // const updateNetwork = useUpdateNetwork(({ data }: { data: { network_name?: string; network_icon_url?: string; }; }) => ({
    //     path: {
    //         network_identity,
    //     },
    //     contentType: "application/json; charset=utf-8",
    //     data,
    // }));

    return (
        <div>
            <Tabs>
                <div class="flex justify-between items-end">
                    <div class="flex items-center gap-2 px-1 pb-2 pt-1">
                        <Show when={network()?.network_icon_url}>
                            {icon => <img src={icon()} alt={network()?.network_name} class="size-4 aspect-square rounded-full" />}
                        </Show>
                        <div class="flex items-baseline gap-1">
                            <span>{network()?.network_name}</span>
                            <span class="text-muted text-xs">
                                #
                                {network()?.network_identity?.toString()}
                            </span>
                        </div>
                    </div>

                    <div class="flex justify-end">
                        <Tabs.List class="flex gap-1">
                            <For each={[{ value: "endpoints", label: "Endpoints" }, { value: "details", label: "Details" }]}>
                                {item => (
                                    <Tabs.Trigger value={item.value} class={button({ variant: "outline", size: "small", class: "border-b-0 !rounded-b-none text-sm" })}>{item.label}</Tabs.Trigger>
                                )}
                            </For>
                        </Tabs.List>
                    </div>
                </div>
                <div class="bg-surface p-4 rounded-md w-full">
                    <Tabs.Content value="endpoints">
                        <NetworkEndpoints network_identity={network_identity} />
                    </Tabs.Content>
                    <Tabs.Content value="details">
                        <div class="w-full space-y-2">
                            <div class="flex gap-2 w-full">
                                <div class="grow">
                                    <div>Name</div>
                                    <input
                                      type="text"
                                      class="input w-full"
                                      value={network()?.network_name}
                                    //   onChange={e => updateNetwork.mutate({ data: { network_name: e.target.value } })}
                                    />
                                </div>
                                <div>
                                    <div>Network Id</div>
                                    <input
                                      type="text"
                                      class="input w-full"
                                      value={network()?.network_identity?.toString()}
                                      disabled
                                    />
                                </div>
                            </div>

                            <div class="space-y-1 w-full">
                                <div>Icon</div>
                                <div class="space-y-2">
                                    <div class="input w-full flex items-center gap-3 px-3">
                                        <div class="size-8 shrink-0 flex items-center justify-center rounded-full bg-surface-alt overflow-hidden">
                                            <Show when={network()?.network_icon_url}>
                                                {icon => <img src={icon()} alt={network()?.network_name} class="size-8 aspect-square rounded-full" />}
                                            </Show>
                                        </div>
                                        <input
                                          type="text"
                                          class="bg-transparent outline-none w-full min-w-0"
                                          value={network()?.network_icon_url}
                                        //   onChange={e => setNetworkIconUrl(e.target.value)}
                                        />
                                    </div>
                                </div>
                            </div>

                            <div class="flex w-full justify-end gap-2">
                                <button
                                  class={button({ variant: "primary" })}
                                  disabled
                                //   disabled={!isDirty()}
                                //   onClick={() => updateNetwork.mutate({ data: { network_name: networkName(), network_icon_url: networkIconUrl() } })}
                                >
                                    Save
                                </button>
                                <NetworkDelete network_identity={network_identity} />
                            </div>
                        </div>
                    </Tabs.Content>
                </div>
            </Tabs>
        </div>
    );
};
