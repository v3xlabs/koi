import { FiCopy, FiExternalLink } from "solid-icons/fi";
import { Component, createMemo, For, JSX, Show } from "solid-js";

import { useVendors, VendorFlag } from "#/api/vendor";
import { addressToHue, truncateAddress } from "#/utils/address";

import { Modal } from "../dialog";
import { NetworkIcon } from "../net/icon";

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
        const links: { flag: VendorFlag; network_identity: number; link: string; }[] = [];

        for (const flag of vendors() ?? []) {
            for (const network_identity of props.networks) {
                const link = EXPLORER_ADDRESS_LINKS[flag as VendorFlag]?.[network_identity]?.replace("$address", props.address);

                if (link) {
                    links.push({ flag, network_identity, link });
                }
            }
        }

        return links;
    });

    return (
        <Modal>
            <Modal.Trigger class={props.class}>
                {props.children ?? "Link"}
            </Modal.Trigger>
            <Modal.Portal>
                <Modal.Overlay />
                <div class="fixed inset-0 z-10">
                    <Modal.Content class="w-full max-w-xl bg-surface rounded-md relative mx-auto mt-24">
                        <Modal.CloseButton />
                        <Modal.Title>Link External</Modal.Title>
                        <div class="p-4 space-y-4">
                            <For each={links()}>
                                {link => (
                                    <a href={link.link} target="_blank" class="hover:underline">
                                        <div class="flex items-center gap-2">
                                            <NetworkIcon network_identity={link.network_identity} />
                                            <span>
                                                {link.flag}
                                            </span>
                                        </div>
                                        <div class="wrap-anywhere">
                                            {link.link}
                                        </div>
                                    </a>
                                )}
                            </For>
                        </div>
                    </Modal.Content>
                </div>
            </Modal.Portal>
        </Modal>
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
