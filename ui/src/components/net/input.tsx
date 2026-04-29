import { Select } from "@kobalte/core/select";
import { FiCheck, FiChevronDown, FiX } from "solid-icons/fi";
import { Accessor, Component, createMemo, For, Show } from "solid-js";

import { useNetworks } from "#/api/network";

import { NetworkIcon } from "./icon";

export type NetworkSelectProperties = {
    value?: Accessor<number[] | null>;
    onChange?: (value: number[] | null) => void;
    multiple?: boolean;
    disabled?: boolean;
};

export const NetworkSelect: Component<NetworkSelectProperties> = (props) => {
    const networksQuery = useNetworks();
    const value = createMemo(() => props.value?.() || []);
    const setValue = props.onChange;

    const allNetworks = createMemo(() => networksQuery.data?.networks ?? []);
    const options = createMemo(() => networksQuery.data?.networks.map(network => network.network_identity) ?? []);

    return (
        <Select<number>
          multiple
          value={value()}
          onChange={setValue}
          options={options()}
          placeholder="Select networks..."
          itemComponent={props => (
                <Select.Item item={props.item} class="flex items-center gap-2 hover:bg-surface-alt cursor-pointer">
                    <NetworkIcon network_id={props.item.rawValue} />

                    <Select.ItemLabel>
                        <Show when={allNetworks().find(network => network.network_identity === props.item.rawValue)} fallback={props.item.rawValue}>
                            {network => (
                                <span>
                                    {network().network_name}
                                    <span class="text-muted text-xs">
                                        #
                                        {network().network_identity}
                                    </span>
                                </span>
                            )}
                        </Show>
                    </Select.ItemLabel>
                    <Select.ItemIndicator>
                        <FiCheck />
                    </Select.ItemIndicator>
                </Select.Item>
            )}
        >
            <Select.Trigger aria-label="Networks" as="div" class="bg-surface border border-border rounded-md px-2 py-1 outline-none w-full max-w-md flex justify-between items-center">
                <Select.Value<string>>
                    {state => (
                        <>
                            <div class="flex items-center gap-1 flex-wrap">
                                <For each={state.selectedOptions()}>
                                    {option => (
                                        <span onPointerDown={e => e.stopPropagation()} class="flex items-center gap-2 border border-border rounded-md px-2 py-1">
                                            <NetworkIcon network_id={Number(option)} />
                                            <Show when={allNetworks().find(network => network.network_identity === Number(option))} fallback={option}>
                                                {network => (
                                                    <span>
                                                        {network().network_name}
                                                        <span class="text-muted text-xs">
                                                            #
                                                            {network().network_identity}
                                                        </span>
                                                    </span>
                                                )}
                                            </Show>
                                            <button onClick={() => state.remove(option)}>
                                                <FiX />
                                            </button>
                                        </span>
                                    )}
                                </For>
                            </div>
                        </>
                    )}
                </Select.Value>
                <Select.Icon>
                    <FiChevronDown />
                </Select.Icon>
            </Select.Trigger>
            <Select.Portal>
                <Select.Content class="bg-surface p-3 rounded-md border border-border outline-none w-full max-w-md">
                    <Select.Listbox />
                </Select.Content>
            </Select.Portal>
        </Select>
    );
};
