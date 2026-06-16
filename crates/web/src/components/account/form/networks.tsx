import { AnyFieldApi } from "@tanstack/solid-form";
import { Accessor, Component, Show } from "solid-js";

import { NetworkSelect } from "#/components/net/input";

type FormNetworkFieldProps = {
    field: Accessor<AnyFieldApi>;
    label: string;
};

export const FormNetworkField: Component<FormNetworkFieldProps> = (props) => {
    const state = () => props.field().state;

    return (
        <label class="space-y-1 block">
            <span class="block">{props.label}</span>
            <NetworkSelect
              value={() => state().value as number[]}
              onChange={value => props.field().handleChange(value ?? [])}
            />
            <Show when={state().meta.isTouched && state().meta.errors.length > 0}>
                <span class="text-sm text-red-500">
                    {state().meta.errors.join(", ")}
                </span>
            </Show>
        </label>
    );
};
