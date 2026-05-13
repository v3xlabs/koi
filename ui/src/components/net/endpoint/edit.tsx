import { Component, createMemo, createSignal, Show } from "solid-js";

import { NetworkEndpoint, useNetworkEndpointStatus, useUpdateNetworkEndpoint } from "#/api/network";
import { button } from "#/components/input/button";
import { narrow } from "#/utils/narrow";

import { NetworkEndpointDelete } from "./delete";

export const NetworkEndpointItem: Component<{ network_identity: number; endpoint: NetworkEndpoint; }> = ({ network_identity, endpoint }) => {
    const updateNetworkEndpoint = useUpdateNetworkEndpoint(({ data }: { data: NetworkEndpoint; }) => ({
        path: {
            network_identity,
            endpoint_identity: endpoint.endpoint_identity ?? "",
        },
        contentType: "application/json; charset=utf-8",
        data,
    }));

    const [label, setLabel] = createSignal(endpoint.endpoint_label);
    const [url, setUrl] = createSignal(endpoint.endpoint_url);
    const [type, setType] = createSignal(endpoint.endpoint_type);
    const [disabled, setDisabled] = createSignal(endpoint.endpoint_disabled);

    const isDirty = createMemo(() =>
        label() !== endpoint.endpoint_label
        || url() !== endpoint.endpoint_url
        || type() !== endpoint.endpoint_type
        || disabled() !== endpoint.endpoint_disabled,
    );

    const status = useNetworkEndpointStatus(() => ({
        path: {
            network_identity,
            endpoint_identity: endpoint.endpoint_identity ?? "",
        },
    }));

    return (
        <li class="space-y-2 p-4">
            <div class="space-y-2">
                <div class="flex gap-2 w-full">
                    <label class="w-full">
                        <span class="text-sm">Label</span>
                        <input
                          type="text"
                          value={label()}
                          onChange={e => setLabel(e.target.value)}
                          class="input w-full"
                        />
                    </label>
                    <label class="flex items-center gap-2 pt-5">
                        <span class="text-sm">Enabled</span>
                        <input
                          type="checkbox"
                          checked={!disabled()}
                          onChange={e => setDisabled(!e.target.checked)}
                          class="checkbox"
                        />
                    </label>
                </div>
                <div class="flex gap-2 w-full">
                    <label class="w-full max-w-xs">
                        <span class="text-sm">Type</span>
                        <input
                          type="text"
                          value={type()}
                          onChange={e => setType(e.target.value)}
                          class="input w-full"
                        />
                    </label>
                    <label class="w-full">
                        <span class="text-sm">URL</span>
                        <input
                          type="text"
                          value={url()}
                          onChange={e => setUrl(e.target.value)}
                          class="input w-full"
                        />
                    </label>
                </div>
            </div>
            <Show when={narrow(() => status.data, x => x.status === "Dead")}>
                {data => (
                    <div class="text-sm text-primary border border-primary rounded-md p-2 text-start">
                        <span>{data().error}</span>
                    </div>
                )}
            </Show>
            <div class="flex justify-end gap-2">
                <button
                  class={button({ variant: "primary" })}
                  disabled={!isDirty()}
                  onClick={() => updateNetworkEndpoint.mutate({
                        data: {
                            endpoint_identity: endpoint.endpoint_identity,
                            network_identity,
                            endpoint_label: label(),
                            endpoint_type: type(),
                            endpoint_url: url(),
                            endpoint_disabled: disabled(),
                        },
                    })}
                >
                    Update
                </button>
                <NetworkEndpointDelete network_identity={network_identity} endpoint_identity={endpoint.endpoint_identity} />
            </div>
        </li>
    );
};
