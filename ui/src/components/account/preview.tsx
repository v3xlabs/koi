import { Component, createMemo, For, Show, Suspense } from "solid-js";

import { useAccount, useAccountBalances } from "#/api/account";
import { useDisplayCurrency } from "#/api/context";
import { truncateAddress } from "#/utils/address";
import { narrow } from "#/utils/narrow";

import { AssetAmount } from "../asset/amount";
import { NetworkIcon } from "../net/icon";
import { AccountIcon } from "./icon";
import { AccountTypeIcon } from "./type";

export type AccountPreviewProperties = {
    account_identity: number;
};

const InlineBalance: Component<{ account_identity: number; }> = (props) => {
    const { displayCurrency } = useDisplayCurrency();
    const accountQuery = useAccountBalances(() => ({ path: { account_identity: props.account_identity }, query: { display_currency: displayCurrency() } }));

    const totalBalance = createMemo(() => accountQuery.data?.total_quote ?? 0);

    return (
        <div>
            <AssetAmount amount={() => BigInt(totalBalance())} asset={displayCurrency} />
        </div>
    );
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
                                <div class="flex items-center gap-2">
                                    <AccountTypeIcon type={() => acc().metadata.type} />
                                    <div>
                                        {acc().name}
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
                            <div>

                                <div>
                                    <Suspense>
                                        <InlineBalance account_identity={account_identity} />
                                    </Suspense>
                                </div>
                                <div class="text-muted text-sm">
                                    <div class="flex items-center gap-1.5 text-muted">
                                    </div>

                                    <Show when={acc().networks.length > 0}>
                                        <ul class="flex items-center justify-end gap-1">
                                            <For each={acc().networks}>
                                                {network => (
                                                    <div class="text-muted text-sm">
                                                        <NetworkIcon network_identity={network} />
                                                    </div>
                                                )}
                                            </For>
                                        </ul>
                                    </Show>
                                </div>
                            </div>
                        </div>
                    </div>
                )}
            </Show>
        </div>
    );
};
