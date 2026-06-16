import { createForm } from "@tanstack/solid-form";
import { createFileRoute } from "@tanstack/solid-router";
import { createMemo, For, Show } from "solid-js";

import { FormNetworkField } from "#/components/account/form/networks";
import { button } from "#/components/input/button";
import { FormNumberField, FormTextField } from "#/components/input/field";

export const Route = createFileRoute("/acc/new/safe")({
    component: () => {
        const form = createForm(() => ({
            defaultValues: {
                name: "",
                networks: [] as number[],
                threshold: 1,
                owners: [""] as string[],
            },
            onSubmit: async () => {
                // Safe deployment is not yet implemented.
            },
        }));

        const canSubmit = createMemo(() => {
            const state = form.state;
            const validOwners = state.values.owners.filter(owner => owner.length >= 42);

            return state.values.name.length > 0
              && state.values.networks.length > 0
              && state.values.threshold > 0
              && state.values.threshold <= validOwners.length
              && validOwners.length > 0;
        });

        return (
            <div class="p-4 mx-auto w-full max-w-lg">
                <div class="text-xl mb-4">
                    New Safe
                </div>
                <form
                  class="bg-surface p-4 rounded-md w-full space-y-4"
                  onSubmit={(event) => {
                        event.preventDefault();
                        event.stopPropagation();
                        form.handleSubmit();
                    }}
                >
                    <div class="text-sm text-yellow-500 bg-yellow-500/10 p-3 rounded">
                        Safe deployment is not yet implemented. This form is a preview of the creation flow.
                    </div>
                    <form.Field name="name">
                        {field => <FormTextField field={field} label="Name" placeholder="My Safe" />}
                    </form.Field>
                    <form.Field name="networks">
                        {field => <FormNetworkField field={field} label="Networks" />}
                    </form.Field>
                    <form.Field name="threshold">
                        {field => (
                            <FormNumberField
                              field={field}
                              label="Threshold"
                              min={1}
                              max={form.state.values.owners.length}
                            />
                        )}
                    </form.Field>
                    <div class="space-y-2">
                        <span class="block">Owners</span>
                        <For each={form.state.values.owners}>
                            {(_, index) => (
                                <form.Field name={`owners[${index()}]`}>
                                    {field => (
                                        <div class="flex gap-2">
                                            <FormTextField
                                              field={field}
                                              label={`Owner ${index() + 1}`}
                                              placeholder="0x..."
                                            />
                                            <button
                                              type="button"
                                              class={button({ variant: "danger", size: "small" })}
                                              onClick={() => {
                                                    const next = form.state.values.owners.filter((__, i) => i !== index());

                                                    form.setFieldValue("owners", next);
                                                }}
                                            >
                                                Remove
                                            </button>
                                        </div>
                                    )}
                                </form.Field>
                            )}
                        </For>
                        <button
                          type="button"
                          class={button({ variant: "secondary" })}
                          onClick={() => {
                                form.setFieldValue("owners", [...form.state.values.owners, ""]);
                            }}
                        >
                            Add Owner
                        </button>
                    </div>
                    <Show when={form.state.values.threshold > form.state.values.owners.filter(owner => owner.length >= 42).length}>
                        <div class="text-sm text-red-500">
                            Threshold cannot be greater than the number of valid owners.
                        </div>
                    </Show>
                    <div class="flex justify-end">
                        <button type="submit" class={button({ variant: "primary" })} disabled={!canSubmit()}>
                            Deploy Safe
                        </button>
                    </div>
                </form>
            </div>
        );
    },
});
