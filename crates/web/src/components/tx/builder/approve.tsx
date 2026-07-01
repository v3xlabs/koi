import { createForm } from "@tanstack/solid-form";
import { createEffect, createSignal, Show, useContext } from "solid-js";
import { formatUnits, isAddress } from "viem";

import { appcontext } from "#/api";
import { useAccountAssetBalance } from "#/api/account";
import { useAsset } from "#/api/asset";
import { FormAddressField } from "#/components/account/form/address";
import { FormAmountField, FormAssetSelectField } from "#/components/input/field";
import { Toggle } from "#/components/input/toggle";

type BuilderData = {
    token: string;
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
    const { displayCurrency: [displayCurrency] } = useContext(appcontext);
    const [unlimited, setUnlimited] = createSignal(props.data.amount === "unlimited");

    const form = createForm(() => ({
        defaultValues: {
            token: props.data.token ?? "",
            spender: props.data.spender ?? "",
            amount: unlimited() ? "" : (props.data.amount ?? ""),
        } as BuilderData,
        onSubmit: async () => {},
    }));

    createEffect(() => {
        const values = form.state.values;
        const cleaned: Partial<BuilderData> = {};

        if (values.token) cleaned.token = values.token;

        if (values.spender) cleaned.spender = values.spender;

        if (unlimited()) {
            cleaned.amount = "unlimited";
        }
        else if (values.amount) {
            cleaned.amount = values.amount;
        }

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
            <div class="text-lg font-bold">Approve</div>
            <form class="space-y-4">
                <form.Field name="token">
                    {field => <FormAssetSelectField field={field} label="Token" networkIdentity={props.networkIdentity} />}
                </form.Field>
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
                <Show when={!unlimited()}>
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
                </Show>
            </form>
        </div>
    );
};
