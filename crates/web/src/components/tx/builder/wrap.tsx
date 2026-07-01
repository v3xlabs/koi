import { createForm } from "@tanstack/solid-form";
import { createEffect, createSignal } from "solid-js";

import { FormCombinedAssetAmountField } from "#/components/input/field";

type BuilderData = {
    asset: string;
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
    const [currentDirection, setCurrentDirection] = createSignal(props.direction);

    const form = createForm(() => ({
        defaultValues: {
            asset: props.data.asset ?? "",
            amount: props.data.amount ?? "",
        } as BuilderData,
        onSubmit: async () => {},
    }));

    createEffect(() => {
        const values = form.state.values;
        const cleaned: Partial<BuilderData> = {};

        if (values.asset) cleaned.asset = values.asset;

        if (values.amount) cleaned.amount = values.amount;

        props.onChange(cleaned);
    });

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
            </form>
        </div>
    );
};
