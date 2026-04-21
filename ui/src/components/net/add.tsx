import { Popover } from "@kobalte/core/popover";
import { createMemo, For } from "solid-js";

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

    return (
        <Popover>
            <Popover.Trigger>
                <button class="btn btn-primary">
                    Add Network
                </button>
            </Popover.Trigger>
            <Popover.Portal>
                <Popover.Content class="bg-surface p-4 rounded-md border border-border outline-none">
                    <div>
                        Hello
                        <ul>
                            <For each={availablePresets()}>
                                {preset => (
                                    <li>
                                        <button class="btn btn-primary" onClick={() => createNetwork.mutate({ data: preset })}>
                                            {preset.network_name}
                                        </button>
                                    </li>
                                )}
                            </For>
                        </ul>
                    </div>
                </Popover.Content>
            </Popover.Portal>
        </Popover>
    );
};
