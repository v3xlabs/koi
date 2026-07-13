import { Component, Show } from "solid-js";

import { useNetwork } from "#/api/network";
import { CachedImage } from "#/utils/image-cache";

export const NetworkIcon: Component<{ network_identity: number; }> = ({ network_identity }) => {
    const networkQuery = useNetwork(() => ({ path: { network_identity } }));

    return (
        <Show when={networkQuery.data?.network_icon_url}>
            {icon => <CachedImage src={icon()} alt={networkQuery.data?.network_name} class="size-4 aspect-square rounded-full" />}
        </Show>
    );
};
