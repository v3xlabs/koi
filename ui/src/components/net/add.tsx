import { Popover } from "@kobalte/core/popover";
import { Tabs } from "@kobalte/core/tabs";
import { createMemo, createSignal, For, Show } from "solid-js";

import { Network, useCreateNetwork, useNetworkPresets, useNetworks } from "#/api/network";

export const NetworkAdd = () => {
    const presetsQuery = useNetworkPresets();
    const networksQuery = useNetworks();
    const availablePresets = createMemo(() => {
        const presets = presetsQuery.data ?? [];
        const networks = networksQuery.data?.networks ?? [];

        return presets.filter(preset => !networks.some(network => network.network_identity === preset.network_identity));
    });
    const createNetwork = useCreateNetwork(({ data }: { data: Network; }) => ({
        contentType: "application/json; charset=utf-8",
        data,
    }));

    const [chainId, setChainId] = createSignal("");
    const [name, setName] = createSignal("");

    return (
        <Popover>
            <Popover.Trigger>
                <button class="btn btn-primary">
                    Add Network
                </button>
            </Popover.Trigger>
            <Popover.Portal>
                <Popover.Content class="bg-surface p-4 rounded-md border border-border outline-none w-full max-w-md">
                    <div class="w-full">
                        <Tabs>
                            <Tabs.List>
                                <Tabs.Trigger value="new" class="btn btn-secondary">
                                    New
                                </Tabs.Trigger>
                                <Tabs.Trigger value="presets" class="btn btn-secondary">
                                    Presets
                                </Tabs.Trigger>
                            </Tabs.List>
                            <Tabs.Content value="new">
                                <div class="w-full">
                                    <label class="space-y-1 block w-full">
                                        <span>Chain Id</span>
                                        <input type="text" class="input w-full" value={chainId()} onChange={e => setChainId(e.target.value)} />
                                    </label>
                                    <label class="space-y-1 block w-full">
                                        <span>Name</span>
                                        <input type="text" class="input w-full" value={name()} onChange={e => setName(e.target.value)} />
                                    </label>
                                    <div class="flex justify-end">
                                        <button class="btn btn-primary" onClick={() => createNetwork.mutate({ data: { network_identity: Number.parseInt(chainId()), network_name: name() } })}>
                                            Create
                                        </button>
                                    </div>
                                </div>
                            </Tabs.Content>
                            <Tabs.Content value="presets">
                                <div class="w-full">
                                    <div>
                                        Presets
                                    </div>
                                    <ul>
                                        <For each={availablePresets()}>
                                            {preset => (
                                                <li class="w-full">
                                                    <button class="btn btn-secondary w-full flex items-center gap-2" onClick={() => createNetwork.mutate({ data: preset })}>
                                                        <Show when={preset.network_icon_url}>
                                                            {icon => <img src={icon()} alt={preset.network_name} class="size-4 aspect-square" />}
                                                        </Show>
                                                        <span>
                                                            {preset.network_name}
                                                        </span>
                                                    </button>
                                                </li>
                                            )}
                                        </For>
                                    </ul>
                                </div>
                            </Tabs.Content>
                        </Tabs>
                    </div>
                </Popover.Content>
            </Popover.Portal>
        </Popover>
    );
};
