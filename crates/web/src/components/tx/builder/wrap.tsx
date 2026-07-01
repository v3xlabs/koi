import { createForm } from "@tanstack/solid-form";
import { createEffect, createSignal, useContext } from "solid-js";
import { formatUnits } from "viem";

import { appcontext } from "#/api";
import { useAccountAssetBalance } from "#/api/account";
import { useAsset } from "#/api/asset";
import { FormAmountField, FormAssetSelectField } from "#/components/input/field";

type BuilderData = {
    token: string;
    amount: string;
};

type Props = {
    direction: "wrap" | "unwrap";
    data: Partial<BuilderData>;
    onChange: (data: Partial<BuilderData>) => void;
    onDirectionChange: (direction: "wrap" | "unwrap") => void;
    accountIdentity: number;
    networkIdentity: number;
};

export const TxWrapBuilder = (props: Props) => {
    const { displayCurrency: [displayCurrency] } = useContext(appcontext);
    const [currentDirection, setCurrentDirection] = createSignal(props.direction);

    const form = createForm(() => ({
        defaultValues: {
            token: props.data.token ?? "",
            amount: props.data.amount ?? "",
        } as BuilderData,
        onSubmit: async () => {},
    }));

    createEffect(() => {
        const values = form.state.values;
        const cleaned: Partial<BuilderData> = {};

        if (values.token) cleaned.token = values.token;

        if (values.amount) cleaned.amount = values.amount;

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
            <div class="text-lg font-bold">Wrap / Unwrap</div>
            <div class="relative inline-flex rounded-md border border-border bg-surface p-1">
                <button
                  type="button"
                  classList={{
                        "relative rounded-sm px-3 py-1.5 text-sm transition-colors cursor-pointer": true,
                        "bg-primary text-primary-foreground": currentDirection() === "wrap",
                        "text-muted hover:text-foreground": currentDirection() !== "wrap",
                    }}
                  onClick={() => {
                        setCurrentDirection("wrap");
                        props.onDirectionChange("wrap");
                    }}
                >
                    Wrap
                </button>
                <button
                  type="button"
                  classList={{
                        "relative rounded-sm px-3 py-1.5 text-sm transition-colors cursor-pointer": true,
                        "bg-primary text-primary-foreground": currentDirection() === "unwrap",
                        "text-muted hover:text-foreground": currentDirection() !== "unwrap",
                    }}
                  onClick={() => {
                        setCurrentDirection("unwrap");
                        props.onDirectionChange("unwrap");
                    }}
                >
                    Unwrap
                </button>
            </div>
            <form class="space-y-4">
                <form.Field name="token">
                    {field => <FormAssetSelectField field={field} label="Token" networkIdentity={props.networkIdentity} />}
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
            </form>
        </div>
    );
};
