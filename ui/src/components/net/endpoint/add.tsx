import { Popover } from "@kobalte/core/popover";
import { Component, createSignal } from "solid-js";

import { NetworkEndpoint, useCreateNetworkEndpoint } from "#/api/network";

export const NetworkEndpointAdd: Component<{ network_id: number; }> = ({ network_id }) => {
    const createNetwork = useCreateNetworkEndpoint(({ data }: { data: NetworkEndpoint; }) => ({
        contentType: "application/json; charset=utf-8",
        data,
        path: {
            network_id,
        },
    }));

    const [name, setName] = createSignal("");
    const [url, setUrl] = createSignal("");
    const [type, setType] = createSignal("http");
    const [priority, setPriority] = createSignal(1);
    const [enabled, setEnabled] = createSignal(false);

    return (
        <Popover>
            <Popover.Trigger>
                <button class="btn btn-primary">
                    Add Endpoint
                </button>
            </Popover.Trigger>
            <Popover.Portal>
                <Popover.Content class="bg-surface p-4 rounded-md border border-border outline-none w-full max-w-md">
                    <div class="w-full">
                        <div class="w-full">
                            <label class="space-y-1 block w-full">
                                <span>Name</span>
                                <input type="text" class="input w-full" value={name()} onChange={e => setName(e.target.value)} />
                            </label>
                            <label class="space-y-1 block w-full">
                                <span>URL</span>
                                <input type="text" class="input w-full" value={url()} onChange={e => setUrl(e.target.value)} />
                            </label>
                            <label class="space-y-1 block w-full">
                                <span>Type</span>
                                <input type="text" class="input w-full" value={type()} onChange={e => setType(e.target.value)} />
                            </label>
                            <label class="space-y-1 block w-full">
                                <span>Priority</span>
                                <input type="number" class="input w-full" value={priority()} onChange={e => setPriority(Number(e.target.value))} />
                            </label>
                            <label class="space-y-1 block w-full">
                                <span>Enabled</span>
                                <input type="checkbox" class="input w-full" checked={enabled()} onChange={e => setEnabled(e.target.checked)} />
                            </label>
                            <div class="flex justify-end">
                                <button
                                  class="btn btn-primary"
                                  onClick={() => createNetwork.mutate({
                                        data: {
                                            endpoint_identity: network_id.toString(),
                                            endpoint_label: name(),
                                            endpoint_type: type(),
                                            endpoint_url: url(),
                                            endpoint_priority: priority(),
                                            endpoint_disabled: enabled(),
                                            network_identity: network_id,
                                        },
                                    })}
                                >
                                    Create
                                </button>
                            </div>
                        </div>
                    </div>
                </Popover.Content>
            </Popover.Portal>
        </Popover>
    );
};
