import { Component, For, Show, Suspense } from "solid-js";

import { useAccounts } from "#/api/account";

import { AccountPreview } from "./preview";

export const AccountEdit: Component = () => {
    const accountsQuery = useAccounts();

    return (
        <div>
            <div>
                Account Edit
            </div>
            <Suspense fallback={<div>Loading...</div>}>
                <Show when={accountsQuery.data}>
                    {data => (
                        <div>
                            <For each={data().accounts}>
                                {account => (
                                    <AccountPreview account_id={account.account_id} />
                                )}
                            </For>
                        </div>
                    )}
                </Show>
            </Suspense>
        </div>
    );
};
