import { createForm } from "@tanstack/solid-form";
import { createFileRoute, useNavigate } from "@tanstack/solid-router";
import { createMemo, createSignal, For, Show } from "solid-js";

import { api } from "#/api";
import { useCreateAccount, useDeriveFromMnemonic } from "#/api/account";
import { FormNetworkField } from "#/components/account/form/networks";
import { button } from "#/components/input/button";
import { FormTextField } from "#/components/input/field";

const SCAN_COUNT = 5;

export const Route = createFileRoute("/acc/_n/import/mnemonic")({
    staticData: {
        title: "Import Mnemonic",
    },
    component: () => {
        const navigate = useNavigate();
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
        const derive = useDeriveFromMnemonic(({ data }: { data: { mnemonic: string; paths: string[]; }; }) => ({
            contentType: "application/json; charset=utf-8",
            data: { mnemonic: data.mnemonic, paths: data.paths },
        }));

        const [derived, setDerived] = createSignal<{ path: string; address: string; }[]>([]);
        const [selected, setSelected] = createSignal<string[]>([]);

        const form = createForm(() => ({
            defaultValues: {
                name: "",
                networks: [] as number[],
                mnemonic: "",
                basePath: "m/44'/60'/0'/0",
            },
            onSubmit: async () => {
                // Multi-account import is handled manually below.
            },
        }));

        const scanPaths = createMemo(() => {
            const base = form.state.values.basePath.replace(/\/$/, "");

            return Array.from({ length: SCAN_COUNT }, (_, index) => `${base}/${index}`);
        });

        const handleScan = async () => {
            const mnemonic = form.state.values.mnemonic;

            if (mnemonic.trim().split(/\s+/).length < 12) return;

            const result = await derive.mutateAsync({ data: { mnemonic, paths: scanPaths() } });

            setDerived(result.results);
            setSelected(result.results.map(item => item.path));
        };

        const handleImport = async () => {
            const name = form.state.values.name;
            const networks = form.state.values.networks;
            const mnemonic = form.state.values.mnemonic;
            const selectedPaths = selected();

            if (name.length === 0 || networks.length === 0 || selectedPaths.length === 0) return;

            for (const path of selectedPaths) {
                const nextIdResponse = await api("/acc/next-id", "get", {});

                if (nextIdResponse.status !== 200) continue;

                const account_identity = nextIdResponse.data;
                const derivedResult = await derive.mutateAsync({ data: { mnemonic, paths: [path] } });
                const address = derivedResult.results[0]?.address;

                if (!address) continue;

                await createAccount.mutateAsync({
                    data: {
                        account_identity,
                        name: `${name} ${path.split("/").pop()}`,
                        networks,
                        display_order: 0,
                        address,
                    },
                });
            }

            navigate({ to: "/" });
        };

        const canScan = createMemo(() => form.state.values.mnemonic.trim().split(/\s+/).length >= 12
          && form.state.values.basePath.length > 0
          && !derive.isPending);

        const canImport = createMemo(() => form.state.values.name.length > 0
          && form.state.values.networks.length > 0
          && selected().length > 0
          && !createAccount.isPending
          && !derive.isPending);

        return (
            <form
              class="bg-surface p-4 rounded-md w-full space-y-4"
              onSubmit={(event) => {
                    event.preventDefault();
                    event.stopPropagation();
                }}
            >
                <form.Field name="mnemonic">
                    {field => (
                        <FormTextField
                          field={field}
                          label="Mnemonic"
                          placeholder="word1 word2 ..."
                        />
                    )}
                </form.Field>
                <form.Field name="basePath">
                    {field => <FormTextField field={field} label="Base Derivation Path" />}
                </form.Field>
                <div class="flex justify-end">
                    <button
                      type="button"
                      class={button({ variant: "secondary" })}
                      onClick={handleScan}
                      disabled={!canScan()}
                    >
                        Scan
                        {" "}
                        {SCAN_COUNT}
                        {" "}
                        Accounts
                    </button>
                </div>
                <form.Field name="name">
                    {field => <FormTextField field={field} label="Account Name Prefix" placeholder="My account" />}
                </form.Field>
                <form.Field name="networks">
                    {field => <FormNetworkField field={field} label="Networks" />}
                </form.Field>
                <Show when={derived().length > 0}>
                    <div class="space-y-2">
                        <span class="block">Select accounts to import</span>
                        <For each={derived()}>
                            {(item) => {
                                const isSelected = () => selected().includes(item.path);

                                return (
                                    <label class="flex items-center gap-3 p-2 rounded bg-surface-alt cursor-pointer">
                                        <input
                                          type="checkbox"
                                          checked={isSelected()}
                                          onChange={(event) => {
                                                if (event.target.checked) {
                                                    setSelected(prev => [...prev, item.path]);
                                                }
                                                else {
                                                    setSelected(prev => prev.filter(path => path !== item.path));
                                                }
                                            }}
                                        />
                                        <div class="min-w-0">
                                            <div class="font-mono text-sm truncate">{item.path}</div>
                                            <div class="text-muted text-xs truncate">{item.address}</div>
                                        </div>
                                    </label>
                                );
                            }}
                        </For>
                    </div>
                </Show>
                <Show when={derive.error || createAccount.error}>
                    <div class="text-sm text-red-500">
                        {derive.error?.message || createAccount.error?.message}
                    </div>
                </Show>
                <div class="flex justify-end">
                    <button
                      type="button"
                      class={button({ variant: "primary" })}
                      onClick={handleImport}
                      disabled={!canImport()}
                    >
                        Import Selected
                    </button>
                </div>
            </form>
        );
    },
});
