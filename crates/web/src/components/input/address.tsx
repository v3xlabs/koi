import { Accessor, Component, JSX } from "solid-js";

export const ERC3770_IDENTIFIERS = {
    eth: 1,
    sep: 11_155_111,
    base: 8453,
    op: 10,
    arb: 42_161,
    bsc: 56,
};

export type AddressInputProperties = {
    value?: Accessor<string>;
    onChange?: (value: string) => void;
    // when set, if the input starts with a safe{wallet} identifier, like `eth:0x123...456` the network identifier will be passed to the function
    // ERC-3770
    onNetworkDetected?: (network_identifier: number) => void;
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
