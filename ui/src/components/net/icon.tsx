import { Component, Show } from "solid-js";

import { useNetwork } from "#/api/network";

export const NetworkIcon: Component<{ network_id: number; }> = ({ network_id }) => {
    const networkQuery = useNetwork(() => ({ path: { network_id } }));

    return (
        <Show when={networkQuery.data?.network_icon_url}>
            {icon => <img src={icon()} alt={networkQuery.data?.network_name} class="size-4 aspect-square rounded-full" />}
        </Show>
    );
};
