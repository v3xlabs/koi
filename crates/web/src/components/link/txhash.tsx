import { FiCopy, FiExternalLink } from "solid-icons/fi";
import { Component, createMemo, JSX, Show } from "solid-js";

import { useVendors, VendorFlag } from "#/api/vendor";
import { addressToHue, truncateAddress } from "#/utils/address";

import { explorerKeyFromFlag, explorerNameFromFlag, ExplorerLinksModal } from "./explorer";

const EXPLORER_TXHASH_LINKS: Partial<Record<VendorFlag, Record<number, string>>> = {
    etherscan_link_tx_hash: {
        1: "https://etherscan.io/tx/$txhash",
        11_155_111: "https://sepolia.etherscan.io/tx/$txhash",
    },
    blockscout_link_tx_hash: {
        1: "https://eth.blockscout.com/tx/$txhash",
        11_155_111: "https://sepolia.blockscout.com/tx/$txhash",
    },
};

export const TxHashExternalLinkModal: Component<{ txhash: string; network_identity: number; children?: JSX.Element; class?: string; }> = ({ txhash, network_identity, children, class: className }) => {
    const vendorsQuery = useVendors();
    const vendors = createMemo(() => vendorsQuery.data?.vendors);

    const links = createMemo(() => {
        const items: { flag: VendorFlag; link: string; }[] = [];

        for (const flag of vendors() ?? []) {
            const link = EXPLORER_TXHASH_LINKS[flag as VendorFlag]?.[network_identity]?.replace("$txhash", txhash);

            if (link) {
                items.push({ flag, link });
            }
        }

        return items
            .flatMap((item) => {
                const explorerKey = explorerKeyFromFlag(item.flag);

                if (!explorerKey) return [];

                return [{
                    link: item.link,
                    network_identity,
                    explorerName: explorerNameFromFlag(item.flag),
                    explorerKey,
                }];
            })
            .toSorted((a, b) => a.explorerName.localeCompare(b.explorerName));
    });

    return (
        <ExplorerLinksModal
          title="Open in explorer"
          description="You're about to open this transaction on an external block explorer. Review the destination URL before continuing."
          class={className}
          emptyMessage="No block explorer links are enabled for this transaction."
          links={links}
        >
            {children ?? "Link"}
        </ExplorerLinksModal>
    );
};

export const TxHashPreview: Component<{
    txhash: string;
    network_identity: number;
    truncate?: boolean;
    actions?: boolean;
    class?: string;
}> = (props) => {
    const hue = createMemo(() => addressToHue(props.txhash));
    const display = createMemo(() => (props.truncate === false ? props.txhash : truncateAddress(props.txhash)));

    const copy = async (event: MouseEvent) => {
        event.stopPropagation();
        await navigator.clipboard.writeText(props.txhash);
    };

    return (
        <span
          class={`inline-flex max-w-full items-center gap-1 rounded-md border px-1.5 py-0.5 align-middle text-xs ${props.class ?? ""}`}
          style={{
                "background-color": `hsl(${hue()} 55% 45% / 0.13)`,
                "border-color": `hsl(${hue()} 55% 45% / 0.35)`,
                "color": `hsl(${hue()} 70% 78%)`,
            }}
          title={props.txhash}
        >
            <code class="truncate bg-transparent p-0 text-[inherit]">
                {display()}
            </code>
            <Show when={props.actions !== false}>
                <button
                  type="button"
                  class="rounded p-0.5 text-[inherit] opacity-70 hover:bg-foreground/10 hover:opacity-100"
                  title="Copy transaction hash"
                  onClick={copy}
                >
                    <FiCopy class="size-3" />
                </button>
                <TxHashExternalLinkModal
                  txhash={props.txhash}
                  network_identity={props.network_identity}
                  class="rounded p-0.5 text-[inherit] opacity-70 hover:bg-foreground/10 hover:opacity-100"
                >
                  <span onClick={event => event.stopPropagation()}>
                    <FiExternalLink class="size-3" />
                  </span>
                </TxHashExternalLinkModal>
            </Show>
        </span>
    );
};
