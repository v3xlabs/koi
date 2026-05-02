import { Popover } from "@kobalte/core/popover";
import { Tabs } from "@kobalte/core/tabs";
import { createMemo, createSignal, For, Show, Suspense } from "solid-js";

import { Network, useCreateNetwork, useNetworkPresets, useNetworks } from "#/api/network";

import { NetworkIconSuggestions } from "./discovery";

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

    const [networkIdentity, setNetworkId] = createSignal<number | undefined>(undefined);
    const [name, setName] = createSignal("");
    const [iconUrl, setIconUrl] = createSignal("");
    const canCreate = createMemo((network_identity = networkIdentity()) => network_identity && name().length > 0);

    return (
        <Popover
          placement="bottom-end"
          gutter={8}
        >
            <Popover.Trigger>
                <button class="btn btn-primary">
                    Add Network
                </button>
            </Popover.Trigger>
            <Popover.Anchor />
            <Popover.Portal>
                <Popover.Content class="bg-surface p-4 rounded-md border border-border outline-none w-full max-w-md popover-content z-10">
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
                                        <input
                                          type="text"
                                          class="input w-full"
                                          value={networkIdentity()?.toString() ?? ""}
                                          placeholder="1"
                                          onChange={e => setNetworkId(Number.parseInt(e.target.value) ?? undefined)}
                                        />
                                    </label>
                                    <label class="space-y-1 block w-full">
                                        <span>Name</span>
                                        <input
                                          type="text"
                                          class="input w-full"
                                          value={name()}
                                          onChange={e => setName(e.target.value)}
                                        />
                                    </label>
                                    <label class="space-y-1 w-full">
                                        <span>Icon</span>
                                        <div class="space-y-2">
                                            <div class="input w-full flex items-center gap-3 px-3">
                                                <div class="size-8 shrink-0 flex items-center justify-center rounded-full bg-surface-alt overflow-hidden">
                                                    <Show when={iconUrl()}>
                                                        {icon => <img src={icon()} alt={name() || "Network icon"} class="size-6 aspect-square rounded-full" />}
                                                    </Show>
                                                </div>
                                                <input
                                                  type="text"
                                                  class="bg-transparent outline-none w-full min-w-0"
                                                  value={iconUrl()}
                                                  onChange={e => setIconUrl(e.target.value)}
                                                  placeholder="https://example.com/icon.png"
                                                />
                                            </div>
                                            <Show when={networkIdentity()}>
                                                {network_identity => (
                                                    <Suspense>
                                                        <NetworkIconSuggestions
                                                          network_identity={network_identity}
                                                          selected_icon_url={iconUrl}
                                                          onSelect={setIconUrl}
                                                        />
                                                    </Suspense>
                                                )}
                                            </Show>
                                        </div>
                                    </label>
                                    <div class="flex justify-end">
                                        <button class="btn btn-primary" disabled={!canCreate()} onClick={() => createNetwork.mutate({ data: { network_identity: networkIdentity()!, network_name: name(), network_icon_url: iconUrl() || undefined } })}>
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
