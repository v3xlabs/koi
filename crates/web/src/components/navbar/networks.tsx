import { Popover } from "@kobalte/core/popover";
import { Link } from "@tanstack/solid-router";
import { FaSolidNetworkWired } from "solid-icons/fa";
import { For, Show } from "solid-js";

import { useNetworks } from "#/api/network";
import { button } from "#/components/input/button";
import { CachedImage } from "#/utils/image-cache";

export const NetworkWidget = () => {
    const networksQuery = useNetworks();

    return (
        <Popover>
            <Popover.Trigger class="nav-icon-button">
                <FaSolidNetworkWired />
            </Popover.Trigger>
            <Popover.Content class="popover-content p-3 w-full max-w-md">
                <ul class="divide-y divide-border">
                    <For each={networksQuery.data?.networks}>
                        {network => (
                            <li>
                                <button class={button({ variant: "ghost", class: "w-full justify-start text-sm" })}>
                                    <Show when={network.network_icon_url}>
                                        {icon => <CachedImage src={icon()} alt={network.network_name} class="size-4 aspect-square rounded-full" />}
                                    </Show>
                                    {network.network_name}
                                </button>
                            </li>
                        )}
                    </For>
                </ul>
                <Popover.CloseButton class="mt-3 w-full">
                    <Link to="/settings/networks" class={button({ variant: "primary", class: "w-full text-sm" })}>
                        Configure
                    </Link>
                </Popover.CloseButton>
            </Popover.Content>
        </Popover>
    );
};
