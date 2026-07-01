import { createForm } from "@tanstack/solid-form";
import { createEffect, For } from "solid-js";

import { FormAmountField, FormAssetSelectField, FormCombinedAssetAmountField } from "#/components/input/field";

type BuilderData = {
    assetIn: string;
    assetOut: string;
    amountIn: string;
    amountOutMin: string;
    slippage: string;
    provider: string;
};

type Props = {
    data: Partial<BuilderData>;
    onChange: (data: Partial<BuilderData>) => void;
    accountIdentity: number;
    networkIdentity: number;
};

const SWAP_PROVIDERS = [
    { value: "uniswap_v2", label: "Uniswap V2" },
    { value: "uniswap_v3", label: "Uniswap V3" },
] as const;

export const TxSwapBuilder = (props: Props) => {
    const form = createForm(() => ({
        defaultValues: {
            assetIn: props.data.assetIn ?? "",
            assetOut: props.data.assetOut ?? "",
            amountIn: props.data.amountIn ?? "",
            slippage: props.data.slippage ?? "0.5",
            amountOutMin: props.data.amountOutMin ?? "",
            provider: props.data.provider ?? "uniswap_v2",
        } as BuilderData,
        onSubmit: async () => {},
    }));

    createEffect(() => {
        const values = form.state.values;
        const cleaned: Partial<BuilderData> = {};

        if (values.assetIn) cleaned.assetIn = values.assetIn;

        if (values.assetOut) cleaned.assetOut = values.assetOut;

        if (values.amountIn) cleaned.amountIn = values.amountIn;

        if (values.slippage) cleaned.slippage = values.slippage;

        if (values.amountOutMin) cleaned.amountOutMin = values.amountOutMin;

        if (values.provider) cleaned.provider = values.provider;

        props.onChange(cleaned);
    });

    return (
        <div class="space-y-4">
            <div class="text-lg font-bold">Swap</div>
            <form class="space-y-4">
                <form.Field name="assetIn">
                    {assetInField => (
                        <form.Field name="amountIn">
                            {amountInField => (
                                <FormCombinedAssetAmountField
                                  amountField={amountInField}
                                  assetField={assetInField}
                                  label="You pay"
                                  networkIdentity={props.networkIdentity}
                                  accountIdentity={props.accountIdentity}
                                />
                            )}
                        </form.Field>
                    )}
                </form.Field>
                <form.Field name="assetOut">
                    {field => <FormAssetSelectField field={field} label="You receive" networkIdentity={props.networkIdentity} />}
                </form.Field>

                <div class="space-y-1">
                    <label class="space-y-1 block">
                        <span class="block">Provider</span>
                        <select
                          class="input w-full"
                          value={form.state.values.provider}
                          onChange={e => form.setFieldValue("provider", e.currentTarget.value)}
                        >
                            <For each={SWAP_PROVIDERS}>
                                {p => (
                                    <option value={p.value} selected={form.state.values.provider === p.value}>
                                        {p.label}
                                    </option>
                                )}
                            </For>
                        </select>
                    </label>
                </div>
                <form.Field name="slippage">
                    {field => (
                        <FormAmountField
                          field={field}
                          label="Slippage %"
                          placeholder="0.5"
                        />
                    )}
                </form.Field>
            </form>
        </div>
    );
};
