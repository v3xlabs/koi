import { FiCopy, FiExternalLink } from "solid-icons/fi";
import { Component, createMemo, JSX, Show } from "solid-js";

import { useVendors, VendorFlag } from "#/api/vendor";
import { addressToHue, truncateAddress } from "#/utils/address";

import { explorerKeyFromFlag, explorerNameFromFlag, ExplorerLinksModal } from "./explorer";

const EXPLORER_ADDRESS_LINKS: Partial<Record<VendorFlag, Record<number, string>>> = {
    etherscan_link_address: {
        1: "https://etherscan.io/address/$address",
        11_155_111: "https://sepolia.etherscan.io/address/$address",
    },
    blockscout_link_address: {
        1: "https://eth.blockscout.com/address/$address",
        11_155_111: "https://sepolia.blockscout.com/address/$address",
    },
};

export const AddressExternalLinkModal: Component<{ address: string; networks: number[]; children?: JSX.Element; class?: string; }> = (props) => {
    const vendorsQuery = useVendors();
    const vendors = createMemo(() => vendorsQuery.data?.vendors);

    const links = createMemo(() => {
        const items: { flag: VendorFlag; network_identity: number; link: string; }[] = [];

        for (const flag of vendors() ?? []) {
            for (const network_identity of props.networks) {
                const link = EXPLORER_ADDRESS_LINKS[flag as VendorFlag]?.[network_identity]?.replace("$address", props.address);

                if (link) {
                    items.push({ flag, network_identity, link });
                }
            }
        }

        return items
            .flatMap((item) => {
                const explorerKey = explorerKeyFromFlag(item.flag);

                if (!explorerKey) return [];

                return [{
                    link: item.link,
                    network_identity: item.network_identity,
                    explorerName: explorerNameFromFlag(item.flag),
                    explorerKey,
                }];
            })
            .toSorted((a, b) => a.network_identity - b.network_identity || a.explorerName.localeCompare(b.explorerName));
    });

    const hue = createMemo(() => addressToHue(props.address));

    const copyAddress = async () => {
        await navigator.clipboard.writeText(props.address);
    };

    return (
        <ExplorerLinksModal
          title="Open in explorer"
          description="You're about to open this address on an external block explorer. Review the destination URL before continuing."
          class={props.class}
          emptyMessage="No block explorer links are enabled for this address."
          links={links}
        >
            {props.children ?? "Link"}
        </ExplorerLinksModal>
    );
};

export const AddressPreview: Component<{ address: string; network_identity?: number; truncate?: boolean; actions?: boolean; class?: string; }> = (props) => {
    const hue = createMemo(() => addressToHue(props.address));
    const display = createMemo(() => (props.truncate === false ? props.address : truncateAddress(props.address)));
    const copy = async (event: MouseEvent) => {
        event.stopPropagation();
        await navigator.clipboard.writeText(props.address);
    };

    return (
        <span
          class={`inline-flex max-w-full items-center gap-1 rounded-md border px-1.5 py-0.5 align-middle text-xs ${props.class ?? ""}`}
          style={{
                "background-color": `hsl(${hue()} 70% 45% / 0.13)`,
                "border-color": `hsl(${hue()} 70% 45% / 0.35)`,
                "color": `hsl(${hue()} 80% 78%)`,
            }}
          title={props.address}
        >
            <code class="truncate bg-transparent p-0 text-[inherit]">
                {display()}
            </code>
            <Show when={props.actions !== false}>
                <button
                  type="button"
                  class="rounded p-0.5 text-[inherit] opacity-70 hover:bg-foreground/10 hover:opacity-100"
                  title="Copy address"
                  onClick={copy}
                >
                    <FiCopy class="size-3" />
                </button>
                <Show when={props.network_identity}>
                    {network_identity => (
                        <AddressExternalLinkModal
                          address={props.address}
                          networks={[network_identity()]}
                          class="rounded p-0.5 text-[inherit] opacity-70 hover:bg-foreground/10 hover:opacity-100"
                        >
                            <FiExternalLink class="size-3" />
                        </AddressExternalLinkModal>
                    )}
                </Show>
            </Show>
        </span>
    );
};
