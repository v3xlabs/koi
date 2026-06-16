import { AnyFieldApi } from "@tanstack/solid-form";
import { Accessor, Component, Show } from "solid-js";

import { AddressInput } from "#/components/input/address";

type FormAddressFieldProps = {
    field: Accessor<AnyFieldApi>;
    label: string;
    placeholder?: string;
};

export const FormAddressField: Component<FormAddressFieldProps> = (props) => {
    const state = () => props.field().state;

    return (
        <label class="space-y-1 block">
            <span class="block">{props.label}</span>
            <AddressInput
              class="w-full"
              placeholder={props.placeholder}
              value={() => state().value as string}
              onChange={value => props.field().handleChange(value)}
            />
            <Show when={state().meta.isTouched && state().meta.errors.length > 0}>
                <span class="text-sm text-red-500">
                    {state().meta.errors.join(", ")}
                </span>
            </Show>
        </label>
    );
};
