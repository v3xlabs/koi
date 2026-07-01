import { Component, Show } from "solid-js";

import BlockscoutIcon from "#/assets/blockscout.png";
import EtherscanIcon from "#/assets/etherscan.svg";

export type ExplorerKey = "etherscan" | "blockscout";

export const ExplorerIcon: Component<{ explorer: ExplorerKey; class?: string; }> = (props) => {
    const className = () => props.class ?? "size-5";

    return (
        <Show
          when={props.explorer === "etherscan"}
          fallback={<img src={BlockscoutIcon} alt="Blockscout" class={className()} />}
        >
            <img src={EtherscanIcon} alt="Etherscan" class={className()} />
        </Show>
    );
};
