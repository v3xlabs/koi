import { Component, createMemo, For, JSX } from "solid-js";

import { useVendors, VendorFlag } from "#/api/vendor";

import { Modal } from "../dialog";
import { NetworkIcon } from "../net/icon";

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
        const links: { flag: VendorFlag; link: string; }[] = [];

        for (const flag of vendors() ?? []) {
            const link = EXPLORER_TXHASH_LINKS[flag as VendorFlag]?.[network_identity]?.replace("$txhash", txhash);

            if (link) {
                links.push({ flag, link });
            }
        }

        return links;
    });

    return (
        <Modal>
            <Modal.Trigger class={className}>
                {children ?? "Link"}
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
                                            <NetworkIcon network_identity={network_identity} />
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
