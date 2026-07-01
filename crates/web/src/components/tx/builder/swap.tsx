import { createForm } from "@tanstack/solid-form";
import { createEffect, For } from "solid-js";

import { FormAmountField, FormAssetSelectField } from "#/components/input/field";

type BuilderData = {
    tokenIn: string;
    tokenOut: string;
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
            tokenIn: props.data.tokenIn ?? "",
            tokenOut: props.data.tokenOut ?? "",
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

        if (values.tokenIn) cleaned.tokenIn = values.tokenIn;

        if (values.tokenOut) cleaned.tokenOut = values.tokenOut;

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
                <form.Field name="tokenIn">
                    {field => <FormAssetSelectField field={field} label="From Token" networkIdentity={props.networkIdentity} />}
                </form.Field>
                <form.Field name="tokenOut">
                    {field => <FormAssetSelectField field={field} label="To Token" networkIdentity={props.networkIdentity} />}
                </form.Field>
                <form.Field name="amountIn">
                    {field => (
                        <FormAmountField
                          field={field}
                          label="Amount"
                          placeholder="0.0"
                        />
                    )}
                </form.Field>
                <form.Field name="slippage">
                    {field => (
                        <FormAmountField
                          field={field}
                          label="Slippage %"
                          placeholder="0.5"
                        />
                    )}
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
            </form>
        </div>
    );
};
