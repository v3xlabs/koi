import { createForm } from "@tanstack/solid-form";
import { createFileRoute, useNavigate } from "@tanstack/solid-router";
import { Show } from "solid-js";

import { useCreateAccount, useNextAccountId } from "#/api/account";
import { AddressInput } from "#/components/input/address";
import { button } from "#/components/input/button";
import { NetworkSelect } from "#/components/net/input";

export const Route = createFileRoute("/acc/_n/import/safe")({
    staticData: {
        title: "Import Safe",
    },
    component: () => {
        const navigate = useNavigate();
        const nextAccountId = useNextAccountId();
        const createAccount = useCreateAccount(({ data }: { data: { account_identity: number; name: string; networks: number[]; address: string; display_order: number; }; }) => ({
            contentType: "application/json; charset=utf-8",
            data: {
                account_identity: data.account_identity,
                name: data.name,
                networks: data.networks,
                display_order: data.display_order,
                metadata: { type: "safe", evm_address: data.address },
            },
        }));

        const form = createForm(() => ({
            defaultValues: {
                name: "",
                networks: [] as number[],
                address: "",
            },
            onSubmit: async ({ value }) => {
                const account_identity = nextAccountId.data;

                if (!account_identity || account_identity <= 0) return;

                if (value.networks.length === 0) return;

                await createAccount.mutateAsync({
                    data: {
                        account_identity,
                        name: value.name,
                        networks: value.networks,
                        display_order: 0,
                        address: value.address,
                    },
                });

                navigate({ to: "/acc/$account", params: { account: account_identity.toString() } });
            },
        }));

        return (
            <form
              class="bg-surface p-4 rounded-md w-full space-y-4"
              onSubmit={(event) => {
                    event.preventDefault();
                    event.stopPropagation();
                    form.handleSubmit();
                }}
            >
                <form.Field name="name">
                    {field => (
                        <label class="space-y-1 block">
                            <span class="block">Name</span>
                            <input
                              type="text"
                              class="input w-full"
                              placeholder="My Safe"
                              value={field().state.value}
                              onInput={event => field().handleChange(event.currentTarget.value)}
                              onBlur={field().handleBlur}
                            />
                            <Show when={field().state.meta.isTouched && field().state.meta.errors.length > 0}>
                                <span class="text-sm text-red-500">
                                    {field().state.meta.errors.join(", ")}
                                </span>
                            </Show>
                        </label>
                    )}
                </form.Field>
                <form.Field name="address">
                    {field => (
                        <label class="space-y-1 block">
                            <span class="block">Safe Address</span>
                            <AddressInput
                              class="w-full"
                              placeholder="0x..."
                              value={() => field().state.value}
                              onChange={value => field().handleChange(value)}
                              onBlur={field().handleBlur}
                            />
                            <Show when={field().state.meta.isTouched && field().state.meta.errors.length > 0}>
                                <span class="text-sm text-red-500">
                                    {field().state.meta.errors.join(", ")}
                                </span>
                            </Show>
                        </label>
                    )}
                </form.Field>
                <form.Field name="networks">
                    {field => (
                        <label class="space-y-1 block">
                            <span class="block">Networks</span>
                            <NetworkSelect
                              value={() => field().state.value}
                              onChange={value => field().handleChange(value ?? [])}
                            />
                            <Show when={field().state.meta.isTouched && field().state.meta.errors.length > 0}>
                                <span class="text-sm text-red-500">
                                    {field().state.meta.errors.join(", ")}
                                </span>
                            </Show>
                        </label>
                    )}
                </form.Field>
                <Show when={createAccount.error}>
                    <div class="text-sm text-red-500">
                        {createAccount.error?.message}
                    </div>
                </Show>
                <div class="flex justify-end">
                    <form.Subscribe
                      selector={state => ({
                            address: state.values.address,
                            name: state.values.name,
                            networks: state.values.networks,
                        })}
                    >
                        {state => (
                            <button
                              type="submit"
                              class={button({ variant: "primary" })}
                              disabled={state().name.length === 0
                                || state().networks.length === 0
                                || state().address.length < 42
                                || createAccount.isPending
                                || (nextAccountId.data ?? 0) <= 0}
                            >
                                Import
                            </button>
                        )}
                    </form.Subscribe>
                </div>
            </form>
        );
    },
});
