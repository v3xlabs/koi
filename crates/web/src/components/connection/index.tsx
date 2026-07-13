import { Popover } from "@kobalte/core/popover";
import { Select } from "@kobalte/core/select";
import { FaSolidChain } from "solid-icons/fa";
import { FiCheck, FiChevronDown } from "solid-icons/fi";
import { createMemo, createSignal, For, Show } from "solid-js";

import { useAccounts } from "#/api/account";
import {
    useAddConnection,
    useConnections,
    useDisconnectConnection,
    useRemoveConnection,
} from "#/api/connection";
import { useNetworks } from "#/api/network";
import { AccountSwitcherItem } from "#/components/account/switcher-item";
import { button } from "#/components/input/button";
import { NetworkIcon } from "#/components/net/icon";

export const ConnectionButton = () => {
    const [url, setUrl] = createSignal("");
    const [accountIdentity, setAccountIdentity] = createSignal<number | null>(null);
    const [networkIdentity, setNetworkIdentity] = createSignal<number | null>(null);
    const addConnection = useAddConnection(props => ({
        contentType: "application/json; charset=utf-8",
        data: props,
    }));
    const disconnectConnection = useDisconnectConnection(props => ({
        path: { connection_id: props.connection_id },
    }));
    const removeConnection = useRemoveConnection(props => ({
        path: { connection_id: props.connection_id },
    }));
    const accounts = useAccounts();
    const connections = useConnections();
    const networks = useNetworks();

    const networkOptions = createMemo(() => networks.data?.networks.map(network => network.network_identity) ?? []);
    const networkName = (network_identity: number) =>
        networks.data?.networks.find(network => network.network_identity === network_identity)?.network_name
        ?? `Network #${network_identity}`;

    const connect = async () => {
        const account_identity = accountIdentity();
        const network_identity = networkIdentity();

        if (account_identity === null || network_identity === null) return;

        await addConnection.mutateAsync({
            url: url().trim(),
            account_identity,
            network_identity,
        });
        setUrl("");
    };

    const canConnect = () =>
        url().trim().length > 0
        && accountIdentity() !== null
        && networkIdentity() !== null;

    return (
        <Popover>
            <Popover.Trigger class="nav-icon-button relative">
                <FaSolidChain class={(connections.data?.connections.length ?? 0) > 0 ? "text-primary-foreground" : "text-muted"} />
                <Show when={(connections.data?.connections.length ?? 0) > 0}>
                    <div class="absolute bottom-[-0.3em] right-[-0.3em] text-muted text-xs bg-surface-alt rounded-full px-1.5 py-0.5 flex items-center justify-center">
                        {connections.data?.connections.length}
                    </div>
                </Show>
            </Popover.Trigger>
            <Popover.Portal>
                <Popover.Content class="popover-content p-3 w-full max-w-md">
                    <div class="space-y-2">
                        <input
                          type="text"
                          class="input w-full"
                          value={url()}
                          onChange={e => setUrl(e.target.value)}
                          placeholder="openlv://..."
                        />
                        <div class="grid gap-2">
                            <div class="max-h-40 overflow-y-auto rounded-md border border-border bg-surface p-1">
                                <Show
                                  when={(accounts.data?.accounts.length ?? 0) > 0}
                                  fallback={<div class="px-2 py-4 text-center text-sm text-muted">No accounts yet.</div>}
                                >
                                    <For each={accounts.data?.accounts ?? []}>
                                        {account => (
                                            <button
                                              type="button"
                                              class="flex w-full items-center gap-2 rounded-md px-2 py-2 text-left outline-none transition-colors hover:bg-surface-alt focus-visible:ring-2 focus-visible:ring-primary/50"
                                              classList={{
                                                    "bg-primary/10 ring-1 ring-primary/50": accountIdentity() === account.account_identity,
                                                }}
                                              aria-pressed={accountIdentity() === account.account_identity}
                                              onClick={() => setAccountIdentity(account.account_identity)}
                                            >
                                                <AccountSwitcherItem account_identity={account.account_identity} />
                                                <Show when={accountIdentity() === account.account_identity}>
                                                    <FiCheck class="shrink-0 text-primary" />
                                                </Show>
                                            </button>
                                        )}
                                    </For>
                                </Show>
                            </div>
                            <Select<number>
                              value={networkIdentity()}
                              onChange={setNetworkIdentity}
                              options={networkOptions()}
                              placeholder="Network"
                              itemComponent={props => (
                                    <Select.Item item={props.item} class="select__item">
                                        <div class="flex items-center gap-2">
                                            <NetworkIcon network_identity={props.item.rawValue} />
                                            <Select.ItemLabel>
                                                <Show
                                                  when={networks.data?.networks.find(network => network.network_identity === props.item.rawValue)}
                                                  fallback={`Network #${props.item.rawValue}`}
                                                >
                                                    {network => network().network_name}
                                                </Show>
                                            </Select.ItemLabel>
                                        </div>
                                        <Select.ItemIndicator class="select__item-indicator">
                                            <FiCheck />
                                        </Select.ItemIndicator>
                                    </Select.Item>
                                )}
                            >
                                <Select.Trigger class="select__trigger w-full min-h-9">
                                    <Select.Value<number>>
                                        {state => (
                                            <Show when={state.selectedOption()} fallback={<span class="text-muted">Network</span>}>
                                                {network_identity => (
                                                    <span class="select__single-value">
                                                        <NetworkIcon network_identity={network_identity()} />
                                                        {networkName(network_identity())}
                                                    </span>
                                                )}
                                            </Show>
                                        )}
                                    </Select.Value>
                                    <Select.Icon>
                                        <FiChevronDown />
                                    </Select.Icon>
                                </Select.Trigger>
                                <Select.Content class="select__content w-full max-w-md">
                                    <Select.Listbox />
                                </Select.Content>
                            </Select>
                        </div>
                        <div class="flex justify-end">
                            <button
                              class={button({ variant: "primary" })}
                              disabled={addConnection.isPending || !canConnect()}
                              onClick={connect}
                            >
                                Connect
                            </button>
                        </div>
                    </div>
                    <div>
                        <For each={connections.data?.connections ?? []}>
                            {(connection) => {
                                const connection_id = connection.connection_id;

                                return (
                                    <div>
                                        <div>
                                            {connection.connection_id}
                                        </div>
                                        <div>
                                            {connection.status}
                                        </div>
                                        <Show when={connection.status !== "disconnected"}>
                                            <button
                                              class={button({ variant: "secondary" })}
                                              disabled={disconnectConnection.isPending}
                                              onClick={() => disconnectConnection.mutate({ connection_id })}
                                            >
                                                Disconnect
                                            </button>
                                        </Show>
                                        <Show when={connection.status === "disconnected"}>
                                            <button
                                              class={button({ variant: "primary" })}
                                              disabled={removeConnection.isPending}
                                              onClick={() => removeConnection.mutate({ connection_id })}
                                            >
                                                Remove
                                            </button>
                                        </Show>
                                    </div>
                                );
                            }}
                        </For>
                    </div>
                </Popover.Content>
            </Popover.Portal>
        </Popover>
    );
};
