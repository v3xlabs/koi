import { Modal } from "#/components/dialog";
import { Component } from "solid-js";

type ReceiveProperties = {

}

export const Receive: Component<ReceiveProperties> = (props) => {

    return (
        <Modal>
            <Modal.Trigger>
                hello
            </Modal.Trigger>
            <Modal.Portal>
                <Modal.Overlay />
                <div class="fixed inset-0">
                    <Modal.Content class="w-full max-w-md bg-surface rounded-md p-4 relative mx-auto mt-24">
                        <Modal.CloseButton />
                        <Modal.Title>
                            Title
                        </Modal.Title>
                    </Modal.Content>
                </div>
            </Modal.Portal>
        </Modal>
    )
};
