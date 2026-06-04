import { Component, Show, Suspense } from "solid-js";

import { useAccount } from "#/api/account";
import { truncateAddress } from "#/utils/address";
import { narrow } from "#/utils/narrow";

import { AccountBalance } from "./balance";
import { AccountIcon } from "./icon";
import { AccountTypeIcon } from "./type";

export const AccountSwitcherItem: Component<{ account_identity: number; }> = (props) => {
    const account = useAccount(() => ({ path: { account_identity: props.account_identity } }));

    return (
        <Show when={account.data}>
            {acc => (
                <div class="flex min-w-0 items-center gap-2">
                    <div class="size-9 shrink-0 rounded-md bg-surface-alt">
                        <Show when={narrow(() => acc().metadata, x => "evm_address" in x)}>
                            {metadata => <AccountIcon address={() => metadata().evm_address} />}
                        </Show>
                    </div>
                    <div class="min-w-0 grow leading-none">
                        <div class="flex items-center gap-1.5 text-sm font-medium leading-none">
                            <span class="truncate">
                                {acc().name}
                            </span>
                            <AccountTypeIcon type={() => acc().metadata.type} />
                        </div>
                        <Show when={narrow(() => acc().metadata, x => "evm_address" in x)}>
                            {metadata => (
                                <div class="truncate text-sm leading-none text-muted">
                                    {truncateAddress(metadata().evm_address)}
                                </div>
                            )}
                        </Show>
                    </div>
                    <div class="shrink-0 text-sm leading-none">
                        <Suspense>
                            <AccountBalance account_identity={props.account_identity} />
                        </Suspense>
                    </div>
                </div>
            )}
        </Show>
    );
};
