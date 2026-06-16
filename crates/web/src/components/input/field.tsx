import { AnyFieldApi } from "@tanstack/solid-form";
import { Accessor, Component, JSX, Show } from "solid-js";

type FormFieldProps = {
    field: Accessor<AnyFieldApi>;
    label: string;
    children: (field: Accessor<AnyFieldApi>) => JSX.Element;
};

export const FormField: Component<FormFieldProps> = (props) => {
    const state = () => props.field().state;

    return (
        <label class="space-y-1 block">
            <span class="block">{props.label}</span>
            {props.children(props.field)}
            <Show when={state().meta.isTouched && state().meta.errors.length > 0}>
                <span class="text-sm text-red-500">
                    {state().meta.errors.join(", ")}
                </span>
            </Show>
        </label>
    );
};

type FormTextFieldProps = {
    field: Accessor<AnyFieldApi>;
    label: string;
    type?: string;
    placeholder?: string;
};

export const FormTextField: Component<FormTextFieldProps> = props => (
    <FormField field={props.field} label={props.label}>
        {field => (
            <input
              type={props.type ?? "text"}
              class="input w-full"
              placeholder={props.placeholder}
              value={field().state.value as string}
              onInput={e => field().handleChange(e.target.value)}
              onBlur={field().handleBlur}
            />
        )}
    </FormField>
);

type FormTextAreaFieldProps = {
    field: Accessor<AnyFieldApi>;
    label: string;
    placeholder?: string;
    rows?: number;
};

export const FormTextAreaField: Component<FormTextAreaFieldProps> = props => (
    <FormField field={props.field} label={props.label}>
        {field => (
            <textarea
              class="input w-full"
              placeholder={props.placeholder}
              rows={props.rows ?? 4}
              value={field().state.value as string}
              onInput={e => field().handleChange(e.target.value)}
              onBlur={field().handleBlur}
            />
        )}
    </FormField>
);

type FormNumberFieldProps = {
    field: Accessor<AnyFieldApi>;
    label: string;
    min?: number;
    max?: number;
};

export const FormNumberField: Component<FormNumberFieldProps> = props => (
    <FormField field={props.field} label={props.label}>
        {field => (
            <input
              type="number"
              class="input w-full"
              min={props.min}
              max={props.max}
              value={field().state.value as number}
              onInput={e => field().handleChange(Number.parseInt(e.target.value) || 0)}
              onBlur={field().handleBlur}
            />
        )}
    </FormField>
);
