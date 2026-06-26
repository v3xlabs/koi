import { createForm } from "@tanstack/solid-form";
import { createEffect, useContext } from "solid-js";
import { formatUnits, isAddress } from "viem";

import { appcontext } from "#/api";
import { useAccountAssetBalance } from "#/api/account";
import { useAsset } from "#/api/asset";
import { FormAddressField } from "#/components/account/form/address";
import { FormAmountField, FormAssetSelectField, FormTextAreaField } from "#/components/input/field";

type BuilderData = {
    to: string;
    token: string;
    amount: string;
    data: string;
};

type Props = {
    data: Partial<BuilderData>;
    onChange: (data: Partial<BuilderData>) => void;
    accountIdentity: number;
    networkIdentity: number;
};

export const TxSendBuilder = (props: Props) => {
    const { displayCurrency: [displayCurrency] } = useContext(appcontext);

    const form = createForm(() => ({
        defaultValues: {
            to: props.data.to ?? "",
            token: props.data.token ?? "",
            amount: props.data.amount ?? "",
            data: props.data.data ?? "",
        } as BuilderData,
        onSubmit: async () => {},
    }));

    createEffect(() => {
        const values = form.state.values;
        const cleaned: Partial<BuilderData> = {};

        if (values.to) cleaned.to = values.to;

        if (values.token) cleaned.token = values.token;

        if (values.amount) cleaned.amount = values.amount;

        if (values.data) cleaned.data = values.data;

        props.onChange(cleaned);
    });

    const tokenIdentity = () => form.state.values.token || "native:1";
    const hasToken = () => !!form.state.values.token;

    const tokenQuery = useAsset(
        () => ({ path: { asset_identity: tokenIdentity() } }),
        { enabled: hasToken() },
    );
    const balanceQuery = useAccountAssetBalance(
        () => ({
            path: { account_identity: props.accountIdentity, asset_identity: tokenIdentity() },
            query: { display_currency: displayCurrency() },
        }),
        { enabled: hasToken() },
    );

    const balanceHuman = () => {
        const b = balanceQuery.data?.balance;
        const d = tokenQuery.data?.asset_decimals;
        const s = tokenQuery.data?.asset_symbol;

        if (!b || d === undefined) return undefined;

        return { value: formatUnits(BigInt(b), d), symbol: s };
    };

    return (
        <div class="space-y-4">
            <div class="text-lg font-bold">Send</div>
            <form class="space-y-4">
                <form.Field name="token">
                    {field => <FormAssetSelectField field={field} label="Token" networkIdentity={props.networkIdentity} />}
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
                <form.Field name="amount">
                    {field => (
                        <FormAmountField
                          field={field}
                          label="Amount"
                          placeholder="0.0"
                          balance={balanceHuman()?.value}
                          balanceSymbol={balanceHuman()?.symbol}
                        />
                    )}
                </form.Field>
                {/* <form.Field name="data">
                    {field => (
                        <FormTextAreaField
                          field={field}
                          label="Calldata (optional)"
                          placeholder="0x..."
                          rows={3}
                        />
                    )}
                </form.Field> */}
            </form>
        </div>
    );
};
