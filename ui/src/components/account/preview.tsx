import { Component, createMemo, Show } from "solid-js";

import { useAccount } from "#/api/account";
import { truncateAddress } from "#/utils/address";

import { AccountIcon } from "./icon";

export type AccountPreviewProperties = {
    account_id: number;
};

export const AccountPreview: Component<AccountPreviewProperties> = (props) => {
    const account = createMemo(() => useAccount(props.account_id));

    return (
        <div>
            <Show when={account()}>
                {acc => (
                    <div class="flex items-center gap-2">
                        <AccountIcon address={() => acc().evm_address} class="w-8 h-8" />
                        <div>
                            <div>
                                {truncateAddress(acc().evm_address)}
                            </div>
                        </div>
                    </div>
                )}
            </Show>
        </div>
    );
};
