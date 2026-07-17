import { createForm } from "@tanstack/solid-form";
import { createEffect, createSignal, Show } from "solid-js";
import { isAddress } from "viem";

import { FormAddressField } from "#/components/account/form/address";
import { FormAssetSelectField, FormCombinedAssetAmountField } from "#/components/input/field";
import { Toggle } from "#/components/input/toggle";

type BuilderData = {
    asset: string;
    spender: string;
    amount: string;
};

type Props = {
    data: Partial<BuilderData>;
    onChange: (data: Partial<BuilderData>) => void;
    accountIdentity: number;
    networkIdentity: number;
};

export const TxApproveBuilder = (props: Props) => {
    const [unlimited, setUnlimited] = createSignal(props.data.amount === "unlimited");

    const form = createForm(() => ({
        defaultValues: {
            asset: props.data.asset ?? "",
            spender: props.data.spender ?? "",
            amount: unlimited() ? "" : (props.data.amount ?? ""),
        } as BuilderData,
        onSubmit: async () => {},
    }));

    createEffect(() => {
        const values = form.state.values;
        const cleaned: Partial<BuilderData> = {};

        if (values.asset) cleaned.asset = values.asset;

        if (values.spender) cleaned.spender = values.spender;

        if (unlimited()) {
            cleaned.amount = "unlimited";
        }
        else if (values.amount) {
            cleaned.amount = values.amount;
        }

        props.onChange(cleaned);
    });

    return (
        <div class="space-y-4">
            <div class="text-lg font-bold">Approve</div>
            <form class="space-y-4">
                <form.Field name="asset">
                    {assetField => (
                        <>
                            <Show when={!unlimited()}>
                                <form.Field name="amount">
                                    {amountField => (
                                        <FormCombinedAssetAmountField
                                          amountField={amountField}
                                          assetField={assetField}
                                          label="Asset & Amount"
                                          networkIdentity={props.networkIdentity}
                                          accountIdentity={props.accountIdentity}
                                        />
                                    )}
                                </form.Field>
                            </Show>
                            <Show when={unlimited()}>
                                <FormAssetSelectField
                                  field={assetField}
                                  label="Asset"
                                  networkIdentity={props.networkIdentity}
                                />
                            </Show>
                        </>
                    )}
                </form.Field>
                <div class="flex items-center justify-between">
                    <span class="text-sm font-medium">Unlimited approval</span>
                    <Toggle
                      value={unlimited}
                      onChange={(v) => {
                            setUnlimited(v);

                            if (v) form.setFieldValue("amount", "");
                        }}
                    />
                </div>
                <form.Field
                  name="spender"
                  validators={{
                        onChange: ({ value }) => (!value || isAddress(value) ? undefined : "Invalid address"),
                    }}
                >
                    {field => (
                        <FormAddressField
                          field={field}
                          label="Spender"
                          placeholder="0x..."
                        />
                    )}
                </form.Field>
            </form>
        </div>
    );
};
