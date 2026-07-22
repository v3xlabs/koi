import { Popover } from "@kobalte/core/popover";
import { FiPlus } from "solid-icons/fi";
import { Component, createSignal } from "solid-js";

import { NetworkEndpointCreate, useCreateNetworkEndpoint } from "#/api/network";
import { button } from "#/components/input/button";
import { Toggle } from "#/components/input/toggle";

import { endpointTypeForUrl } from "./connection";
import { NetworkEndpointRoutingMenu } from "./routing";

export const NetworkEndpointAdd: Component<{ network_identity: number; }> = ({ network_identity }) => {
    const createNetwork = useCreateNetworkEndpoint(({ data }: { data: NetworkEndpointCreate; }) => ({
        contentType: "application/json; charset=utf-8",
        data,
        path: {
            network_identity,
        },
    }));

    const [name, setName] = createSignal("");
    const [url, setUrl] = createSignal("");
    const [disabled, setDisabled] = createSignal(false);

    return (
        <Popover>
            <Popover.Trigger class="flex items-center gap-2">
                <button class={button({ variant: "secondary", square: true })}>
                    <FiPlus />
                </button>
            </Popover.Trigger>
            <Popover.Portal>
                <Popover.Content class="popover-content p-4 w-full max-w-md">
                    <div class="space-y-4">
                        <Toggle value={() => !disabled()} onChange={enabled => setDisabled(!enabled)} label="Use endpoint" />
                        <label class="space-y-1 block w-full">
                            <span>Label</span>
                            <input
                              type="text"
                              class="input w-full"
                              value={name()}
                              onChange={e => setName(e.target.value)}
                            />
                        </label>
                        <label class="space-y-1 block w-full">
                            <span>URL</span>
                            <div class="flex gap-1">
                                <input
                                  type="text"
                                  class="input min-w-0 flex-1"
                                  value={url()}
                                  onChange={e => setUrl(e.target.value)}
                                />
                                <NetworkEndpointRoutingMenu />
                            </div>
                        </label>
                            <div class="flex justify-end">
                                <button
                                  class={button({ variant: "primary" })}
                                  disabled={endpointTypeForUrl(url()) === undefined}
                                  onClick={() => createNetwork.mutate({
                                        data: {
                                            endpoint_label: name(),
                                            endpoint_type: endpointTypeForUrl(url()) ?? "http",
                                            endpoint_url: url(),
                                            endpoint_disabled: disabled(),
                                        },
                                    })}
                                >
                                    Create
                                </button>
                            </div>
                    </div>
                </Popover.Content>
            </Popover.Portal>
        </Popover>
    );
};
