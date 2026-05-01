import { Component, JSX } from "solid-js";

import { useDeleteNetworkEndpoint } from "#/api/network";

import { Modal } from "../../dialog";

export const NetworkEndpointDelete: Component<{ network_identity: number; endpoint_identity: number; children?: JSX.Element; }> = ({ network_identity, endpoint_identity, children }) => {
    const deleteNetworkEndpoint = useDeleteNetworkEndpoint(() => ({
        path: {
            network_identity,
            endpoint_identity,
        },
    }));

    return (
        <Modal>
            <Modal.Trigger class={children ? "" : "btn btn-danger"}>
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
                            <button class="btn btn-primary" onClick={() => deleteNetworkEndpoint.mutate({})}>Delete</button>
                            <Modal.CloseButton class="btn btn-secondary">
                                Cancel
                            </Modal.CloseButton>
                        </div>
                    </Modal.Content>
                </div>
            </Modal.Portal>
        </Modal>
    );
};
