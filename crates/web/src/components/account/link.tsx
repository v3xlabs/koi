import { Component, createMemo, JSX, Show } from "solid-js";

import { useAccount } from "#/api/account";

import { AddressExternalLinkModal } from "../link/address";

export const AccountExternalLinkModal: Component<{ account_identity: number; children?: JSX.Element; class?: string; }> = (props) => {
    const accountQuery = useAccount(() => ({ path: { account_identity: props.account_identity } }));
    const account = createMemo(() => accountQuery.data);
    const evmAddress = createMemo(() => {
        const data = account()?.metadata;

        if (data && "evm_address" in data) {
            return data.evm_address;
        }

        return "";
    });
    const networks = createMemo(() => account()?.networks ?? []);

    return (
        <Show when={evmAddress()}>
            {evmAddress => (
                <AddressExternalLinkModal address={evmAddress()} networks={networks()} class={props.class}>
                    {props.children}
                </AddressExternalLinkModal>
            )}
        </Show>
    );
};
