import { Component, JSX } from "solid-js";

import { useDeleteNetwork } from "#/api/network";

import { Modal } from "../dialog";

export const NetworkDelete: Component<{ network_id: number; children?: JSX.Element; }> = ({ network_id, children }) => {
    const deleteNetwork = useDeleteNetwork(() => ({
        path: {
            network_id,
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
                            Delete Network #
                            {network_id.toString()}
                        </Modal.Title>
                        <div class="px-4 pt-4">
                            You are about to delete the network
                            <span class="font-bold bg-surface-alt rounded-md p-1">
                                {network_id}
                            </span>
                            . This action cannot be undone.
                        </div>
                        <div class="w-full flex justify-end gap-2 p-4">
                            <button class="btn btn-primary" onClick={() => deleteNetwork.mutate({})}>Delete</button>
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
