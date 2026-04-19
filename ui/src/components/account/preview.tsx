import { Accessor, Component, Show } from "solid-js";

import { useAccount } from "#/api/account";
import { truncateAddress } from "#/utils/address";

import { AccountIcon } from "./icon";

export type AccountPreviewProperties = {
    account_id: string;
};

const narrow = <A, B extends A>(accessor: Accessor<A>, guard: (v: A) => v is B): B | null => {
    const val = accessor();

    if (guard(val)) {
        return val;
    }

    return null;
};

export const AccountPreview: Component<AccountPreviewProperties> = (props) => {
    const accountQuery = useAccount(props.account_id);

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
                        <div class="w-full">
                            <div class="flex justify-between gap-1">
                                <div>
                                    {acc().name}
                                </div>
                                <div>
                                    {acc().metadata.type}
                                </div>
                            </div>
                            <div class="text-muted text-sm">
                                <Show when={narrow(() => acc().metadata, x => "evm_address" in x)}>
                                    {
                                        x => truncateAddress(x().evm_address)
                                    }
                                </Show>
                            </div>
                        </div>
                    </div>
                )}
            </Show>
        </div>
    );
};
