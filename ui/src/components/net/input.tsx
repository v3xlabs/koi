import { Select } from "@kobalte/core/select";
import { FiCheck, FiChevronDown, FiX } from "solid-icons/fi";
import { Accessor, Component, createMemo, createSignal, For, Show } from "solid-js";

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
    const multiple = createMemo(() => props.multiple ?? true);
    const [open, setOpen] = createSignal(false);

    const allNetworks = createMemo(() => networksQuery.data?.networks ?? []);
    const options = createMemo(() => networksQuery.data?.networks.map(network => network.network_identity) ?? []);
    const selectedNetwork = createMemo(() => allNetworks().find(network => network.network_identity === value()[0]));

    const handleChange = (next: number[]) => {
        if (multiple()) {
            setValue?.(next);

            return;
        }

        const selected = next.find(network_identity => network_identity !== value()[0]) ?? next[0];

        setValue?.(selected ? [selected] : null);
    };

    return (
        <Select<number>
          multiple
          open={open()}
          onOpenChange={setOpen}
          closeOnSelection={!multiple()}
          disallowEmptySelection={!multiple()}
          disabled={props.disabled}
          value={value()}
          onChange={handleChange}
          options={options()}
          placeholder="Select networks..."
          itemComponent={props => (
                <Select.Item item={props.item} class="select__item">
                    <div class="flex items-center gap-2">
                        <NetworkIcon network_identity={props.item.rawValue} />
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
                    </div>
                    <div>
                        <Select.ItemIndicator class="select__item-indicator">
                            <FiCheck />
                        </Select.ItemIndicator>
                    </div>
                </Select.Item>
            )}
        >
            <Show
              when={multiple()}
              fallback={(
                    <Select.Trigger
                      aria-label="Networks"
                      class="select__trigger w-full max-w-md min-h-9"
                      onClick={() => setOpen(true)}
                    >
                        <Select.Value>
                            <Show when={selectedNetwork()} fallback={<span class="text-muted">Select network...</span>}>
                                {network => (
                                    <span class="select__single-value">
                                        <NetworkIcon network_identity={network().network_identity} />
                                        <span>
                                            {network().network_name}
                                            <span class="text-muted text-xs">
                                                #
                                                {network().network_identity}
                                            </span>
                                        </span>
                                    </span>
                                )}
                            </Show>
                        </Select.Value>
                        <div>
                            <Select.Icon>
                                <FiChevronDown />
                            </Select.Icon>
                        </div>
                    </Select.Trigger>
                )}
            >
                <Select.Trigger aria-label="Networks" as="div" class="select__trigger w-full max-w-md min-h-9">
                    <Select.Value<string>>
                        {state => (
                            <div class="flex items-center gap-1 flex-wrap">
                                <For each={state.selectedOptions()}>
                                    {option => (
                                        <span onPointerDown={e => e.stopPropagation()} class="select__token">
                                            <NetworkIcon network_identity={Number(option)} />
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
                                            <button class="select__token-remove" onClick={() => state.remove(option)}>
                                                <FiX />
                                            </button>
                                        </span>
                                    )}
                                </For>
                            </div>
                        )}
                    </Select.Value>
                    <Select.Icon>
                        <FiChevronDown />
                    </Select.Icon>
                </Select.Trigger>
            </Show>
            <Select.Portal>
                <Select.Content class="select__content w-full max-w-md">
                    <Select.Listbox />
                </Select.Content>
            </Select.Portal>
        </Select>
    );
};
