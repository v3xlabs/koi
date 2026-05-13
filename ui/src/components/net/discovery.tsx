import { Accessor, createMemo, For, Show } from "solid-js";

import { useNetworkMetadataDiscovery } from "#/api/network";
import { button } from "#/components/input/button";

export const NetworkIconSuggestions = (props: {
    network_identity: Accessor<number | undefined>;
    onSelect: (iconUrl: string) => void;
    selected_icon_url?: Accessor<string>;
}) => {
    const discoveryQuery = useNetworkMetadataDiscovery(() => ({
        path: { network_identity: props.network_identity() ?? 0 },
    }), {
        enabled: () => !!props.network_identity() && props.network_identity()! > 0,
    });

    const iconSuggestions = createMemo(() => Object.entries(discoveryQuery.data?.options ?? {}).flatMap(([source, option]) => {
        if (!option.icon_url) {
            return [];
        }

        return [[option.icon_url, source] as const];
    }));

    return (
        <Show when={iconSuggestions().length > 0}>
            <div class="space-y-2">
                <div class="text-xs text-muted">
                    Suggestions
                </div>
                <ul class="flex flex-wrap gap-2">
                <For each={iconSuggestions()}>
                    {([iconUrl, source]) => (
                        <li>
                            <button
                              classList={{
                                  "border-primary bg-surface-alt": props.selected_icon_url?.() === iconUrl,
                              }}
                              class={button({ variant: "outline" })}
                              onClick={() => props.onSelect(iconUrl)}
                            >
                                <img src={iconUrl} alt={source} class="size-8 aspect-square rounded-full" />
                                <span class="text-sm capitalize">{source}</span>
                            </button>
                        </li>
                    )}
                </For>
                </ul>
            </div>
        </Show>
    );
};
