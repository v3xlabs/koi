import { Address, blo } from "blo";
import { Accessor, Component, createMemo } from "solid-js";

export type AccountIconProperties = {
    address: Accessor<Address | string>;
    class?: string;
};

export const AccountIcon: Component<AccountIconProperties> = (props) => {
    const hash = createMemo(() => (props.address() == undefined ? "" : blo(props.address() as Address)));

    return (
        <div classList={{
            [props.class ?? ""]: !!props.class,
            "overflow-hidden": true,
        }}
        >
            <img src={hash()} alt="Account Icon" class="w-full h-full" />
        </div>
    );
};
