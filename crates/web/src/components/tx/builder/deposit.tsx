import { createForm } from "@tanstack/solid-form";
import { createEffect, createSignal, useContext } from "solid-js";
import { formatUnits, isAddress } from "viem";

import { appcontext } from "#/api";
import { useAccountAssetBalance } from "#/api/account";
import { useAsset } from "#/api/asset";
import { FormAddressField } from "#/components/account/form/address";
import { FormAmountField, FormAssetSelectField } from "#/components/input/field";

type BuilderData = {
    vault: string;
    amount: string;
};

type Props = {
    direction: "deposit" | "withdraw";
    data: Partial<BuilderData>;
    onChange: (data: Partial<BuilderData>) => void;
    onDirectionChange: (direction: "deposit" | "withdraw") => void;
    accountIdentity: number;
    networkIdentity: number;
};

export const TxDepositBuilder = (props: Props) => {
    const { displayCurrency: [displayCurrency] } = useContext(appcontext);
    const [currentDirection, setCurrentDirection] = createSignal(props.direction);

    const form = createForm(() => ({
        defaultValues: {
            vault: props.data.vault ?? "",
            token: props.data.token ?? "",
            amount: props.data.amount ?? "",
        } as BuilderData & { token: string; },
        onSubmit: async () => {},
    }));

    createEffect(() => {
        const values = form.state.values;
        const cleaned: Partial<BuilderData> = {};

        if (values.vault) cleaned.vault = values.vault;

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
            <div class="text-lg font-bold">Deposit / Withdraw</div>
            <div class="relative inline-flex rounded-md border border-border bg-surface p-1">
                <button
                  type="button"
                  classList={{
                        "relative rounded-sm px-3 py-1.5 text-sm transition-colors cursor-pointer": true,
                        "bg-primary text-primary-foreground": currentDirection() === "deposit",
                        "text-muted hover:text-foreground": currentDirection() !== "deposit",
                    }}
                  onClick={() => {
                        setCurrentDirection("deposit");
                        props.onDirectionChange("deposit");
                    }}
                >
                    Deposit
                </button>
                <button
                  type="button"
                  classList={{
                        "relative rounded-sm px-3 py-1.5 text-sm transition-colors cursor-pointer": true,
                        "bg-primary text-primary-foreground": currentDirection() === "withdraw",
                        "text-muted hover:text-foreground": currentDirection() !== "withdraw",
                    }}
                  onClick={() => {
                        setCurrentDirection("withdraw");
                        props.onDirectionChange("withdraw");
                    }}
                >
                    Withdraw
                </button>
            </div>
            <form class="space-y-4">
                <form.Field name="token">
                    {field => <FormAssetSelectField field={field} label="Vault Token" networkIdentity={props.networkIdentity} />}
                </form.Field>
                <form.Field
                  name="vault"
                  validators={{
                        onChange: ({ value }) => (!value || isAddress(value) ? undefined : "Invalid address"),
                    }}
                >
                    {field => (
                        <FormAddressField
                          field={field}
                          label="Vault Address"
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
            </form>
        </div>
    );
};
