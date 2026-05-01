import { Component, For, Show } from "solid-js";

import { useAccount } from "#/api/account";
import { truncateAddress } from "#/utils/address";
import { narrow } from "#/utils/narrow";

import { NetworkIcon } from "../net/icon";
import { AccountIcon } from "./icon";
import { AccountTypeIcon } from "./type";

export type AccountPreviewProperties = {
    account_identity: number;
};

export const AccountPreview: Component<AccountPreviewProperties> = (props) => {
    const account_identity = props.account_identity;
    const accountQuery = useAccount(() => ({ path: { account_identity } }));

    return (
        <div class="w-full">
            <Show when={accountQuery.data}>
                {acc => (
                    <div class="flex items-center gap-2 w-full">
                        <Show when={narrow(() => acc().metadata, x => "evm_address" in x)}>
                            {
                                x => <AccountIcon address={() => x().evm_address} class="w-8 h-8" />
                            }
                        </Show>
                        <div class="w-full flex justify-between items-center">
                            <div class="">
                                <div>
                                    {acc().name}
                                </div>
                                <div class="text-muted text-sm">
                                    <Show when={narrow(() => acc().metadata, x => "evm_address" in x)}>
                                        {
                                            x => truncateAddress(x().evm_address)
                                        }
                                    </Show>
                                </div>
                            </div>
                            <div class="text-muted text-sm">
                                <div class="flex items-center gap-1.5 text-muted">
                                    {acc().metadata.type}
                                    <AccountTypeIcon type={() => acc().metadata.type} />
                                </div>

                                <Show when={acc().networks.length > 0}>
                                    <ul class="flex items-center justify-end gap-1">
                                        <For each={acc().networks}>
                                            {network => (
                                                <div class="text-muted text-sm">
                                                    <NetworkIcon network_id={network} />
                                                </div>
                                            )}
                                        </For>
                                    </ul>
                                </Show>
                            </div>
                        </div>
                    </div>
                )}
            </Show>
        </div>
    );
};
