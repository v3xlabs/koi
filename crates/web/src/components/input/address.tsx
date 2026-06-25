import { Accessor, Component, JSX, splitProps } from "solid-js";

export type AddressInputProperties = {
    value?: Accessor<string>;
    onChange?: (value: string) => void;
    invalid?: Accessor<boolean> | boolean;
} & Omit<JSX.InputHTMLAttributes<HTMLInputElement>, "value" | "onChange">;

export const AddressInput: Component<AddressInputProperties> = (props) => {
    const [local, inputProps] = splitProps(props, ["value", "onChange", "invalid", "class", "classList"]);
    const invalid = () => typeof local.invalid === "function" ? local.invalid() : !!local.invalid;

    return (
        <input
          {...inputProps}
          type="text"
          value={local.value?.()}
          aria-invalid={invalid() ? "true" : undefined}
          onInput={e => local.onChange?.(e.currentTarget.value)}
          classList={Object.assign({
                input: true,
                [local.class ?? ""]: !!local.class,
            }, local.classList)}
        />
    );
};
