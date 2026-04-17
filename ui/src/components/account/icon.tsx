import { Address, blo } from "blo";
import { Accessor, Component, createMemo } from "solid-js";

export type AccountIconProperties = {
    address: Accessor<Address>;
    class?: string;
};

export const AccountIcon: Component<AccountIconProperties> = (props) => {
    const hash = createMemo(() => (props.address() == undefined ? "" : blo(props.address())));

    return (
        <div class={props.class}>
            <img src={hash()} alt="Account Icon" class="w-full h-full" />
        </div>
    );
};
