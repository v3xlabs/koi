import { createForm } from "@tanstack/solid-form";
import { createEffect } from "solid-js";
import { isAddress } from "viem";

import { FormAddressField } from "#/components/account/form/address";
import { FormCombinedAssetAmountField } from "#/components/input/field";

type BuilderData = {
    to: string;
    asset: string;
    amount: string;
};

type Props = {
    data: Partial<BuilderData>;
    onChange: (data: Partial<BuilderData>) => void;
    accountIdentity: number;
    networkIdentity: number;
};

export const TxSendBuilder = (props: Props) => {
    const form = createForm(() => ({
        defaultValues: {
            to: props.data.to ?? "",
            asset: props.data.asset ?? "",
            amount: props.data.amount ?? "",
        } as BuilderData,
        onSubmit: async () => {},
    }));

    createEffect(() => {
        const values = form.state.values;
        const cleaned: Partial<BuilderData> = {};

        if (values.to) cleaned.to = values.to;

        if (values.asset) cleaned.asset = values.asset;

        if (values.amount) cleaned.amount = values.amount;

        props.onChange(cleaned);
    });

    return (
        <div class="space-y-4">
            <div class="text-lg font-bold">Send</div>
            <form class="space-y-4">
                <form.Field name="asset">
                    {assetField => (
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
                    )}
                </form.Field>
                <form.Field
                  name="to"
                  validators={{
                        onChange: ({ value }) => (!value || isAddress(value) ? undefined : "Invalid address"),
                    }}
                >
                    {field => (
                        <FormAddressField
                          field={field}
                          label="Recipient"
                          placeholder="0x..."
                        />
                    )}
                </form.Field>
            </form>
        </div>
    );
};
