import { Accessor, Component, JSX } from "solid-js";

export type AddressInputProperties = {
    value?: Accessor<string>;
    onChange?: (value: string) => void;
} & Omit<JSX.InputHTMLAttributes<HTMLInputElement>, "value" | "onChange">;

export const AddressInput: Component<AddressInputProperties> = props => (
    <input
      {...props}
      type="text"
      value={props.value?.()}
      onChange={e => props.onChange?.(e.target.value)}
      classList={Object.assign({
            input: true,
            [props.class ?? ""]: !!props.class,
        }, props.classList)}
    />
);
