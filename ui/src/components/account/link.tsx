import { Component, createMemo, For, JSX } from "solid-js";

import { useAccount } from "#/api/account";
import { useVendors, VendorFlag } from "#/api/vendor";

import { Modal } from "../dialog";
import { NetworkIcon } from "../net/icon";

const EXPLORER_ADDRESS_LINKS: Partial<Record<VendorFlag, Record<number, string>>> = {
    etherscan_link_address: {
        1: "https://etherscan.io/address/$address",
    },
    blockscout_link_address: {
        1: "https://eth.blockscout.com/address/$address",
    },
};

export const AccountExternalLinkModal: Component<{ account_identity: number; children?: JSX.Element; class?: string; }> = ({ account_identity, children, class: className }) => {
    const vendorsQuery = useVendors();
    const accountQuery = useAccount(() => ({ path: { account_identity } }));
    const account = createMemo(() => accountQuery.data);
    const vendors = createMemo(() => vendorsQuery.data?.vendors);
    const evmAddress = createMemo(() => {
        const data = account()?.metadata;

        if (data && "evm_address" in data) {
            return data.evm_address;
        }

        return "";
    });

    const links = createMemo(() => {
        const links: { flag: VendorFlag; network_identity: number; link: string; }[] = [];

        for (const flag of vendors() ?? []) {
            for (const network_identity of account()?.networks ?? []) {
                const link = EXPLORER_ADDRESS_LINKS[flag as VendorFlag]?.[network_identity]?.replace("$address", evmAddress());

                if (link) {
                    links.push({ flag, network_identity, link });
                }
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
