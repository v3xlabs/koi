import { createForm } from "@tanstack/solid-form";
import { createEffect, useContext } from "solid-js";
import { formatUnits, isAddress } from "viem";

import { appcontext } from "#/api";
import { useAccountAssetBalance } from "#/api/account";
import { useAsset } from "#/api/asset";
import { FormAddressField } from "#/components/account/form/address";
import { FormAmountField, FormTextAreaField } from "#/components/input/field";

type BuilderData = {
    to: string;
    value: string;
    data: string;
};

type Props = {
    data: Partial<BuilderData>;
    onChange: (data: Partial<BuilderData>) => void;
    accountIdentity: number;
    networkIdentity: number;
};

export const TxRawBuilder = (props: Props) => {
    const { displayCurrency: [displayCurrency] } = useContext(appcontext);

    const form = createForm(() => ({
        defaultValues: {
            to: props.data.to ?? "",
            value: props.data.value ?? "",
            data: props.data.data ?? "",
        } as BuilderData,
        onSubmit: async () => {},
    }));

    createEffect(() => {
        const values = form.state.values;
        const cleaned: Partial<BuilderData> = {};

        if (values.to) cleaned.to = values.to;

        if (values.value) cleaned.value = values.value;

        if (values.data) cleaned.data = values.data;

        props.onChange(cleaned);
    });

    const nativeAssetQuery = useAsset(
        () => ({ path: { asset_identity: `native:${1}` } }),
    );
    const balanceQuery = useAccountAssetBalance(
        () => ({
            path: { account_identity: props.accountIdentity, asset_identity: `native:${1}` },
            query: { display_currency: displayCurrency() },
        }),
    );

    const balanceHuman = () => {
        const b = balanceQuery.data?.balance;
        const d = nativeAssetQuery.data?.asset_decimals;
        const s = nativeAssetQuery.data?.asset_symbol;

        if (!b || d === undefined) return undefined;

        return { value: formatUnits(BigInt(b), d), symbol: s };
    };

    return (
        <div class="space-y-4">
            <div class="text-lg font-bold">Raw Transaction</div>
            <form class="space-y-4">
                <form.Field
                  name="to"
                  validators={{
                        onChange: ({ value }) => (!value || isAddress(value) ? undefined : "Invalid address"),
                    }}
                >
                    {field => (
                        <FormAddressField
                          field={field}
                          label="To"
                          placeholder="0x..."
                        />
                    )}
                </form.Field>
                <form.Field name="value">
                    {field => (
                        <FormAmountField
                          field={field}
                          label="Value (native token)"
                          placeholder="0.0"
                          balance={balanceHuman()?.value}
                          balanceSymbol={balanceHuman()?.symbol}
                        />
                    )}
                </form.Field>
                <form.Field name="data">
                    {field => (
                        <FormTextAreaField
                          field={field}
                          label="Data (hex)"
                          placeholder="0x..."
                          rows={6}
                        />
                    )}
                </form.Field>
            </form>
        </div>
    );
};
