import { createForm } from "@tanstack/solid-form";
import { createEffect, createSignal } from "solid-js";

import { FormCombinedAssetAmountField } from "#/components/input/field";

type BuilderData = {
    vault: string;
    asset: string;
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
    const [currentDirection, setCurrentDirection] = createSignal(props.direction);

    const form = createForm(() => ({
        defaultValues: {
            vault: props.data.vault ?? "",
            asset: props.data.asset ?? "",
            amount: props.data.amount ?? "",
        } as BuilderData,
        onSubmit: async () => {},
    }));

    createEffect(() => {
        const values = form.state.values;
        const cleaned: Partial<BuilderData> = {};

        if (values.vault) cleaned.vault = values.vault;

        if (values.amount) cleaned.amount = values.amount;

        props.onChange(cleaned);
    });

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
                <form.Field name="asset">
                    {assetField => (
                        <form.Field name="amount">
                            {amountField => (
                                <FormCombinedAssetAmountField
                                  amountField={amountField}
                                  assetField={assetField}
                                  label="Vault Asset & Amount"
                                  networkIdentity={props.networkIdentity}
                                  accountIdentity={props.accountIdentity}
                                />
                            )}
                        </form.Field>
                    )}
                </form.Field>
            </form>
        </div>
    );
};
