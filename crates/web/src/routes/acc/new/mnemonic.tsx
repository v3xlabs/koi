import { createForm } from "@tanstack/solid-form";
import { createFileRoute, useNavigate } from "@tanstack/solid-router";
import { createMemo, Show } from "solid-js";

import { useCreateAccount, useDefaultDerivationPath, useDeriveFromMnemonic, useGenerateMnemonic, useNextAccountId } from "#/api/account";
import { FormNetworkField } from "#/components/account/form/networks";
import { button } from "#/components/input/button";
import { FormTextField } from "#/components/input/field";

export const Route = createFileRoute("/acc/new/mnemonic")({
    component: () => {
        const navigate = useNavigate();
        const nextAccountId = useNextAccountId();
        const mnemonic = useGenerateMnemonic();
        const defaultPath = useDefaultDerivationPath();
        const derive = useDeriveFromMnemonic(({ data }: { data: { mnemonic: string; paths: string[]; }; }) => ({
            contentType: "application/json; charset=utf-8",
            data: { mnemonic: data.mnemonic, paths: data.paths },
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
                mnemonic: mnemonic.data?.mnemonic ?? "",
                path: defaultPath.data?.path ?? "m/44'/60'/0'/0/0",
            },
            onSubmit: async ({ value }) => {
                const account_identity = nextAccountId.data;

                if (!account_identity || account_identity <= 0) return;

                if (value.networks.length === 0) return;

                const derived = await derive.mutateAsync({ data: { mnemonic: value.mnemonic, paths: [value.path] } });
                const address = derived.results[0]?.address;

                if (!address) return;

                await createAccount.mutateAsync({
                    data: {
                        account_identity,
                        name: value.name,
                        networks: value.networks,
                        display_order: 0,
                        address,
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
              && state.values.mnemonic.trim().split(/\s+/).length >= 12
              && state.values.path.length > 0
              && !isPending()
              && (nextAccountId.data ?? 0) > 0;
        });

        return (
            <div class="p-4 mx-auto w-full max-w-lg">
                <div class="text-xl mb-4">
                    New Mnemonic
                </div>
                <Show when={mnemonic.data && defaultPath.data} fallback={<div>Loading...</div>}>
                    <form
                      class="bg-surface p-4 rounded-md w-full space-y-4"
                      onSubmit={(event) => {
                            event.preventDefault();
                            event.stopPropagation();
                            form.handleSubmit();
                        }}
                    >
                        <div class="text-sm text-yellow-500 bg-yellow-500/10 p-3 rounded">
                            Write down this mnemonic and store it safely. It will not be shown again and is not stored on the server.
                        </div>
                        <form.Field name="mnemonic">
                            {field => (
                                <FormTextField
                                  field={field}
                                  label="Mnemonic"
                                  placeholder="word1 word2 ..."
                                />
                            )}
                        </form.Field>
                        <form.Field name="name">
                            {field => <FormTextField field={field} label="Name" placeholder="My account" />}
                        </form.Field>
                        <form.Field name="networks">
                            {field => <FormNetworkField field={field} label="Networks" />}
                        </form.Field>
                        <form.Field name="path">
                            {field => <FormTextField field={field} label="Derivation Path" />}
                        </form.Field>
                        <Show when={derive.error || createAccount.error}>
                            <div class="text-sm text-red-500">
                                {derive.error?.message || createAccount.error?.message}
                            </div>
                        </Show>
                        <div class="flex justify-end">
                            <button type="submit" class={button({ variant: "primary" })} disabled={!canSubmit()}>
                                Create
                            </button>
                        </div>
                    </form>
                </Show>
            </div>
        );
    },
});
