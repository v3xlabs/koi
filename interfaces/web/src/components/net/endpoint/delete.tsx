import { Component, JSX } from "solid-js";

import { useDeleteNetworkEndpoint } from "#/api/network";
import { button } from "#/components/input/button";

import { Modal } from "../../dialog";

export const NetworkEndpointDelete: Component<{ network_identity: number; endpoint_identity: number; children?: JSX.Element; }> = ({ network_identity, endpoint_identity, children }) => {
    const deleteNetworkEndpoint = useDeleteNetworkEndpoint<{ network_identity: number; endpoint_identity: number; }>(({ network_identity, endpoint_identity }) => ({
        path: {
            network_identity,
            endpoint_identity,
        },
    }));

    return (
        <Modal>
            <Modal.Trigger class={children ? "" : button({ variant: "danger" })}>
                {children ?? "Delete"}
            </Modal.Trigger>
            <Modal.Portal>
                <Modal.Overlay />
                <div class="fixed inset-0">
                    <Modal.Content class="w-full max-w-xl bg-surface rounded-md relative mx-auto mt-24">
                        <Modal.Title>
                            Delete Network Endpoint #
                            {endpoint_identity}
                        </Modal.Title>
                        <div class="px-4 pt-4">
                            You are about to delete the network endpoint
                            <span class="font-bold bg-surface-alt rounded-md p-1">
                                {endpoint_identity}
                            </span>
                            . This action cannot be undone.
                        </div>
                        <div class="w-full flex justify-end gap-2 p-4">
                            <button class={button({ variant: "primary" })} onClick={() => deleteNetworkEndpoint.mutate({ network_identity, endpoint_identity })}>Delete</button>
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
