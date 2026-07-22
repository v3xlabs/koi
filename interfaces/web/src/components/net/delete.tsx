import { Component, JSX } from "solid-js";

import { useDeleteNetwork } from "#/api/network";
import { button } from "#/components/input/button";

import { Modal } from "../dialog";

export const NetworkDelete: Component<{ network_identity: number; children?: JSX.Element; class?: string; }> = ({ network_identity, children, class: className }) => {
    const deleteNetwork = useDeleteNetwork<{ network_identity: number; }>(({ network_identity }) => ({
        path: {
            network_identity,
        },
    }));

    return (
        <Modal>
            <Modal.Trigger class={className ?? (children ? "" : button({ variant: "danger" }))}>
                {children ?? "Delete"}
            </Modal.Trigger>
            <Modal.Portal>
                <Modal.Overlay />
                <div class="fixed inset-0 z-50">
                    <Modal.Content class="w-full max-w-xl bg-surface rounded-md relative mx-auto mt-24">
                        <Modal.Title>
                            Delete Network #
                            {network_identity.toString()}
                        </Modal.Title>
                        <div class="px-4 pt-4">
                            You are about to delete the network
                            <span class="font-bold bg-surface-alt rounded-md p-1">
                                {network_identity}
                            </span>
                            . This action cannot be undone.
                        </div>
                        <div class="w-full flex justify-end gap-2 p-4">
                            <button class={button({ variant: "primary" })} onClick={() => deleteNetwork.mutate({ network_identity })}>Delete</button>
                            <Modal.CloseButton class={button({ variant: "secondary" })}>
                                Cancel
                            </Modal.CloseButton>
                        </div>
                    </Modal.Content>
                </div>
            </Modal.Portal>
        </Modal>
    );
};
