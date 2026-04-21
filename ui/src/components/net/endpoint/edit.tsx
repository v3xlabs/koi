import { Component, createMemo, createSignal } from "solid-js";

import { NetworkEndpoint, useUpdateNetworkEndpoint } from "#/api/network";

import { NetworkEndpointDelete } from "./delete";

export const NetworkEndpointItem: Component<{ network_id: number; endpoint: NetworkEndpoint; }> = ({ network_id, endpoint }) => {
    const updateNetworkEndpoint = useUpdateNetworkEndpoint(({ data }: { data: NetworkEndpoint; }) => ({
        path: {
            network_id,
            endpoint_id: endpoint.endpoint_identity ?? "",
        },
        contentType: "application/json; charset=utf-8",
        data,
    }));

    const [label, setLabel] = createSignal(endpoint.endpoint_label);
    const [url, setUrl] = createSignal(endpoint.endpoint_url);
    const [type, setType] = createSignal(endpoint.endpoint_type);
    const [priority, setPriority] = createSignal(endpoint.endpoint_priority);
    const [enabled, setEnabled] = createSignal(endpoint.endpoint_disabled);

    const isDirty = createMemo(() =>
        label() !== endpoint.endpoint_label
        || url() !== endpoint.endpoint_url
        || type() !== endpoint.endpoint_type
        || priority() !== endpoint.endpoint_priority
        || enabled() !== endpoint.endpoint_disabled,
    );

    return (
        <li class="space-y-2">
            <div class="space-y-2">
                <div class="flex gap-2 w-full">
                    <label class="w-full">
                        <span class="text-sm">Label</span>
                        <input type="text" value={label()} onChange={e => setLabel(e.target.value)} class="input w-full" />
                    </label>
                    <label class="">
                        <span class="text-sm">Enabled</span>
                        <input type="checkbox" checked={enabled()} onChange={e => setEnabled(e.target.checked)} class="input w-full" />
                    </label>
                </div>
                <div class="flex gap-2 w-full">
                    <label class="w-full max-w-xs">
                        <span class="text-sm">Type</span>
                        <input type="text" value={type()} onChange={e => setType(e.target.value)} class="input w-full" />
                    </label>
                    <label class="w-full">
                        <span class="text-sm">URL</span>
                        <input type="text" value={url()} onChange={e => setUrl(e.target.value)} class="input w-full" />
                    </label>
                </div>
                <label>
                    <span class="text-sm">Priority</span>
                    <input type="number" value={priority()} onChange={e => setPriority(Number(e.target.value))} class="input w-full" />
                </label>
            </div>
            <div class="flex justify-end gap-2">
                <button
                  class="btn btn-primary"
                  disabled={!isDirty()}
                  onClick={() => updateNetworkEndpoint.mutate({
                        data: {
                            endpoint_identity: endpoint.endpoint_identity,
                            network_identity: network_id,
                            endpoint_label: label(),
                            endpoint_type: type(),
                            endpoint_url: url(),
                            endpoint_priority: priority(),
                            endpoint_disabled: enabled(),
                        },
                    })}
                >
                    Update
                </button>
                <NetworkEndpointDelete network_id={network_id} endpoint_id={endpoint.endpoint_identity} />
            </div>
        </li>
    );
};
