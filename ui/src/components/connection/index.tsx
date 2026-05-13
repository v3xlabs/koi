import { Popover } from "@kobalte/core/popover";
import { FaSolidChain } from "solid-icons/fa";
import { createSignal, For, Show } from "solid-js";

import { addConnection, Connection, removeConnection, useConnections } from "#/api/connection";
import { button } from "#/components/input/button";

export const ConnectionButton = () => {
    const [url, setUrl] = createSignal("");
    const connect = async () => {
        addConnection(url(), "4f8b9a49-5de4-4209-b1b9-6b2b5f085463", "1");
    };
    const connections = useConnections();

    return (
        <Popover>
            <Popover.Trigger class="nav-icon-button relative">
                <FaSolidChain class={connections().length > 0 ? "text-primary-foreground" : "text-muted"} />
                <Show when={connections().length > 0}>
                    <div class="absolute bottom-1.5 right-1.5 text-muted text-xs bg-surface-alt rounded-full px-1.5 py-0.5 flex items-center justify-center">
                        {connections().length}
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
                        <div class="flex justify-end">
                            <button class={button({ variant: "primary" })} onClick={connect}>Connect</button>
                        </div>
                    </div>
                    <div>
                        <For each={connections()}>
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
                                              onClick={async () => {
                                                    const connection: Connection = connections().find(c => c.connection_id === connection_id)!;

                                                    if (connection) {
                                                        await connection.session.close();
                                                    }
                                                }}
                                            >
                                                Disconnect
                                            </button>
                                        </Show>
                                        <Show when={connection.status === "disconnected"}>
                                            <button
                                              class={button({ variant: "primary" })}
                                              onClick={() => {
                                                    removeConnection(connection_id);
                                                }}
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
