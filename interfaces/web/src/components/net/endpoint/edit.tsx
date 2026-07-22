import { Collapsible } from "@kobalte/core/collapsible";
import { FiChevronDown } from "solid-icons/fi";
import { Component, createMemo, createSignal, Show } from "solid-js";

import { NetworkEndpoint, NetworkEndpointUpdate, useNetworkEndpointStatus, useUpdateNetworkEndpoint } from "#/api/network";
import { button } from "#/components/input/button";
import { Toggle } from "#/components/input/toggle";
import { narrow } from "#/utils/narrow";

import { endpointTypeForUrl } from "./connection";
import { NetworkEndpointDelete } from "./delete";
import { NetworkEndpointRoutingMenu } from "./routing";
import { NetworkEndpointRpcStats } from "./status";

export const NetworkEndpointItem: Component<{ network_identity: number; endpoint: NetworkEndpoint; }> = ({ network_identity, endpoint }) => {
    const updateNetworkEndpoint = useUpdateNetworkEndpoint(({ data }: { data: NetworkEndpointUpdate; }) => ({
        path: {
            network_identity,
            endpoint_identity: endpoint.endpoint_identity ?? "",
        },
        contentType: "application/json; charset=utf-8",
        data,
    }));

    const [label, setLabel] = createSignal(endpoint.endpoint_label);
    const [url, setUrl] = createSignal(endpoint.endpoint_url);
    const [disabled, setDisabled] = createSignal(endpoint.endpoint_disabled);

    const isDirty = createMemo(() =>
        label() !== endpoint.endpoint_label
        || url() !== endpoint.endpoint_url
        || disabled() !== endpoint.endpoint_disabled,
    );

    const status = useNetworkEndpointStatus(() => ({
        path: {
            network_identity,
            endpoint_identity: endpoint.endpoint_identity ?? "",
        },
    }));

    return (
        <li class="space-y-4 px-4 py-5">
            <Toggle value={() => !disabled()} onChange={enabled => setDisabled(!enabled)} label="Use endpoint" />
            <div class="grid gap-3 sm:grid-cols-2">
                <label class="w-full">
                    <span class="text-sm">Label</span>
                    <input
                      type="text"
                      value={label()}
                      onChange={e => setLabel(e.target.value)}
                      class="input w-full"
                    />
                </label>
                <label class="w-full">
                    <span class="text-sm">URL</span>
                    <div class="flex gap-1">
                        <input
                          type="text"
                          value={url()}
                          onChange={e => setUrl(e.target.value)}
                          class="input min-w-0 flex-1"
                        />
                        <NetworkEndpointRoutingMenu />
                    </div>
                </label>
            </div>
            <Show when={narrow(() => status.data, x => x.status === "Dead")}>
                {data => (
                    <div class="border-l-2 border-primary pl-3 text-sm text-primary">
                        <span>{data().error}</span>
                    </div>
                )}
            </Show>
            <Show when={status.data?.rpc}>
                {rpc => (
                    <Collapsible class="border-t border-border pt-3">
                        <Collapsible.Trigger class="group flex w-full items-center justify-between text-sm text-muted hover:text-foreground">
                            Diagnostics
                            <FiChevronDown class="transition-transform group-data-[expanded]:rotate-180" />
                        </Collapsible.Trigger>
                        <Collapsible.Content class="pt-3">
                            <NetworkEndpointRpcStats stats={rpc} />
                        </Collapsible.Content>
                    </Collapsible>
                )}
            </Show>
            <div class="flex justify-end gap-2">
                <button
                  class={button({ variant: "primary" })}
                  disabled={!isDirty() || endpointTypeForUrl(url()) === undefined}
                  onClick={() => updateNetworkEndpoint.mutate({
                        data: {
                            endpoint_label: label(),
                            endpoint_type: endpointTypeForUrl(url()) ?? endpoint.endpoint_type,
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
