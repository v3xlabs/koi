import { Component, JSX } from "solid-js";

import { useDeleteAccount } from "#/api/account";
import { button } from "#/components/input/button";

import { Modal } from "../dialog";

export const AccountDelete: Component<{ account_identity: number; account_name: string; children?: JSX.Element; onDeleted?: () => void; }> = (props) => {
    const deleteAccount = useDeleteAccount<{ account_identity: number; }>(({ account_identity }) => ({
        path: { account_identity },
    }));

    return (
        <Modal>
            <Modal.Trigger class={props.children ? "" : button({ variant: "danger" })}>
                {props.children ?? "Remove account"}
            </Modal.Trigger>
            <Modal.Portal>
                <Modal.Overlay />
                <Modal.Positioner>
                    <Modal.Content class="w-full max-w-xl bg-surface rounded-md relative mx-auto mt-24">
                        <Modal.CloseButton />
                        <Modal.Title>
                            Remove account
                        </Modal.Title>
                        <div class="px-4 pt-4">
                            You are about to remove
                            {" "}
                            <span class="font-bold bg-surface-alt rounded-md px-1">
                                {props.account_name}
                            </span>
                            . This action cannot be undone.
                        </div>
                        <div class="w-full flex justify-end gap-2 p-4">
                            <button
                              class={button({ variant: "danger" })}
                              onClick={() => deleteAccount.mutate(
                                { account_identity: props.account_identity },
                                { onSuccess: () => props.onDeleted?.() },
                              )}
                            >
                                Remove
                            </button>
                            <Modal.CloseButton class={button({ variant: "secondary" })}>
                                Cancel
                            </Modal.CloseButton>
                        </div>
                    </Modal.Content>
                </Modal.Positioner>
            </Modal.Portal>
        </Modal>
    );
};
