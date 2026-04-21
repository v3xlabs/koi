import { Popover } from "@kobalte/core/popover";
import { For } from "solid-js";

import { useNetworks } from "#/api/network";

export const NetworkWidget = () => {
    const networksQuery = useNetworks();

    return (
        <Popover>
            <Popover.Trigger>
                <button>
                    Networks
                </button>
            </Popover.Trigger>
            <Popover.Content>
                <div class="bg-surface p-3 rounded-md border border-border outline-none w-full max-w-md">
                    <ul class="divide-y divide-border">
                        <For each={networksQuery.data?.networks}>
                            {network => (
                                <li>
                                    <button class="py-1">
                                        {network.network_name}
                                    </button>
                                </li>
                            )}
                        </For>
                    </ul>
                </div>
            </Popover.Content>
        </Popover>
    );
};
