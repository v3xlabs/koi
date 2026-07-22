import { Popover } from "@kobalte/core/popover";
import { Tabs } from "@kobalte/core/tabs";
import { FiMoreVertical } from "solid-icons/fi";
import { Component, createEffect, createMemo, createSignal, For, Show, Suspense } from "solid-js";

import { NetworkUpdate, useNetwork, useNetworkEndpoints, useUpdateNetwork } from "#/api/network";
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
            <ul class="overflow-hidden">
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

const NetworkActions: Component<{ network_identity: number; }> = ({ network_identity }) => (
    <Popover>
        <Popover.Trigger class={button({ variant: "ghost", size: "default", square: true })} aria-label="Network actions">
            <FiMoreVertical />
        </Popover.Trigger>
        <Popover.Portal>
            <Popover.Content class="popover-content w-48 p-1">
                <menu class="flex flex-col">
                    <NetworkDelete
                      network_identity={network_identity}
                      class="rounded-md px-3 py-2 text-left text-sm hover:bg-surface-alt"
                    >
                        Delete network
                    </NetworkDelete>
                </menu>
            </Popover.Content>
        </Popover.Portal>
    </Popover>
);

export const NetworkEdit: Component<{ network_identity: number; }> = (props) => {
    const network_identity = () => props.network_identity;
    const networkQuery = useNetwork(() => ({
        path: {
            network_identity: network_identity(),
        },
    }));
    const network = createMemo(() => networkQuery.data);
    const [activeTab, setActiveTab] = createSignal("endpoints");
    const [networkName, setNetworkName] = createSignal("");
    const [networkIconUrl, setNetworkIconUrl] = createSignal("");
    const updateNetwork = useUpdateNetwork(({ data }: { data: NetworkUpdate; }) => ({
        path: {
            network_identity: network_identity(),
        },
        contentType: "application/json; charset=utf-8",
        data,
    }));
    const isDirty = createMemo(() => (
        networkName() !== network()?.network_name
        || networkIconUrl() !== (network()?.network_icon_url ?? "")
    ));

    createEffect(() => {
        const value = network();

        if (!value) return;

        setNetworkName(value.network_name);
        setNetworkIconUrl(value.network_icon_url ?? "");
    });

    return (
        <div class="overflow-hidden rounded-md bg-surface">
            <Tabs value={activeTab()} onChange={setActiveTab}>
                <div class="flex items-center justify-between gap-2 px-4 pt-4">
                    <div class="flex items-center gap-3">
                        <Show when={network()?.network_icon_url}>
                            {icon => <img src={icon()} alt={network()?.network_name} class="size-8 aspect-square rounded-sm" />}
                        </Show>
                        <div class="flex items-baseline gap-1">
                            <span>{network()?.network_name}</span>
                            <span class="text-muted text-xs">
                                #
                                {network()?.network_identity?.toString()}
                            </span>
                        </div>
                    </div>
                    <div class="flex items-center gap-1">
                        <Show when={isDirty()}>
                            <button
                              class={button({ variant: "primary" })}
                              onClick={() => updateNetwork.mutate({
                                    data: {
                                        network_name: networkName(),
                                        network_icon_url: networkIconUrl() || undefined,
                                    },
                                })}
                            >
                                Save
                            </button>
                        </Show>
                        <Show when={activeTab() === "endpoints"}>
                            <NetworkEndpointAdd network_identity={network_identity()} />
                        </Show>
                        <NetworkActions network_identity={network_identity()} />
                    </div>
                </div>
                <Tabs.List class="mt-3 flex gap-4 border-b border-border px-4">
                            <For each={[{ value: "endpoints", label: "Endpoints" }, { value: "details", label: "Details" }]}>
                                {item => (
                                    <Tabs.Trigger value={item.value} class="border-b-2 border-transparent py-2 text-sm text-muted hover:text-foreground data-[selected]:border-primary data-[selected]:text-foreground">
                                        {item.label}
                                    </Tabs.Trigger>
                                )}
                            </For>
                </Tabs.List>
                <div class="w-full p-4">
                    <Tabs.Content value="endpoints">
                        <NetworkEndpoints network_identity={network_identity()} />
                    </Tabs.Content>
                    <Tabs.Content value="details">
                        <div class="w-full space-y-2">
                            <div class="flex gap-2 w-full">
                                <label class="grow">
                                    <span>Name</span>
                                    <input
                                      type="text"
                                      class="input w-full"
                                      value={networkName()}
                                      onInput={event => setNetworkName(event.currentTarget.value)}
                                    />
                                </label>
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
                                          value={networkIconUrl()}
                                          onInput={event => setNetworkIconUrl(event.currentTarget.value)}
                                        />
                                    </div>
                                </div>
                            </div>

                        </div>
                    </Tabs.Content>
                </div>
            </Tabs>
        </div>
    );
};
