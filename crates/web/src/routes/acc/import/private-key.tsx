import { createForm } from "@tanstack/solid-form";
import { createFileRoute, useNavigate } from "@tanstack/solid-router";
import { createMemo, Show } from "solid-js";

import { useCreateAccount, useDeriveFromPrivateKey, useNextAccountId } from "#/api/account";
import { FormNetworkField } from "#/components/account/form/networks";
import { button } from "#/components/input/button";
import { FormTextField } from "#/components/input/field";

export const Route = createFileRoute("/acc/import/private-key")({
    component: () => {
        const navigate = useNavigate();
        const nextAccountId = useNextAccountId();
        const derive = useDeriveFromPrivateKey(({ private_key }: { private_key: string; }) => ({
            contentType: "application/json; charset=utf-8",
            data: { private_key },
        }));
        const createAccount = useCreateAccount(({ data }: { data: { account_identity: number; name: string; networks: number[]; address: string; display_order: number; }; }) => ({
            contentType: "application/json; charset=utf-8",
            data: {
                account_identity: data.account_identity,
                name: data.name,
                networks: data.networks,
                display_order: data.display_order,
                metadata: { type: "eoa", evm_address: data.address },
            },
        }));

        const form = createForm(() => ({
            defaultValues: {
                name: "",
                networks: [] as number[],
                privateKey: "",
            },
            onSubmit: async ({ value }) => {
                const account_identity = nextAccountId.data;

                if (!account_identity || account_identity <= 0) return;

                if (value.networks.length === 0) return;

                const derived = await derive.mutateAsync({ private_key: value.privateKey });

                await createAccount.mutateAsync({
                    data: {
                        account_identity,
                        name: value.name,
                        networks: value.networks,
                        display_order: 0,
                        address: derived.address,
                    },
                });

                navigate({ to: "/acc/$account", params: { account: account_identity.toString() } });
            },
        }));

        const isPending = createMemo(() => derive.isPending || createAccount.isPending);
        const canSubmit = createMemo(() => {
            const state = form.state;

            return state.values.name.length > 0
              && state.values.networks.length > 0
              && state.values.privateKey.trim().length >= 64
              && !isPending()
              && (nextAccountId.data ?? 0) > 0;
        });

        return (
            <div class="p-4 mx-auto w-full max-w-lg">
                <div class="text-xl mb-4">
                    Import Private Key
                </div>
                <form
                  class="bg-surface p-4 rounded-md w-full space-y-4"
                  onSubmit={(event) => {
                        event.preventDefault();
                        event.stopPropagation();
                        form.handleSubmit();
                    }}
                >
                    <form.Field name="name">
                        {field => <FormTextField field={field} label="Name" placeholder="My account" />}
                    </form.Field>
                    <form.Field name="networks">
                        {field => <FormNetworkField field={field} label="Networks" />}
                    </form.Field>
                    <form.Field name="privateKey">
                        {field => (
                            <FormTextField
                              field={field}
                              label="Private Key"
                              placeholder="0x..."
                            />
                        )}
                    </form.Field>
                    <Show when={derive.error || createAccount.error}>
                        <div class="text-sm text-red-500">
                            {derive.error?.message || createAccount.error?.message}
                        </div>
                    </Show>
                    <div class="flex justify-end">
                        <button type="submit" class={button({ variant: "primary" })} disabled={!canSubmit()}>
                            Import
                        </button>
                    </div>
                </form>
            </div>
        );
    },
});
