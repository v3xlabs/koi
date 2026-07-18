import { Popover } from "@kobalte/core/popover";
import { useNavigate } from "@tanstack/solid-router";
import { FiMoreVertical } from "solid-icons/fi";
import { Component, createSignal } from "solid-js";

import { AccountUpdate, useAccount, useUpdateAccount } from "#/api/account";
import { Modal } from "#/components/dialog";
import { button } from "#/components/input/button";

type AccountRowMenuProps = {
    account_identity: number;
    account_name: string;
};

export const AccountRowMenu: Component<AccountRowMenuProps> = (props) => {
    const navigate = useNavigate();
    const [renameOpen, setRenameOpen] = createSignal(false);
    const [renameValue, setRenameValue] = createSignal(props.account_name);

    const accountQuery = useAccount(() => ({ path: { account_identity: props.account_identity } }));

    const updateAccount = useUpdateAccount(({ data }: { data: AccountUpdate; }) => ({
        path: { account_identity: props.account_identity },
        contentType: "application/json; charset=utf-8",
        data,
    }));

    const saveRename = () => {
        const account = accountQuery.data;

        if (!account) return;

        updateAccount.mutate({
            data: {
                ...account,
                name: renameValue(),
            },
        }, {
            onSuccess: () => setRenameOpen(false),
        });
    };

    return (
        <>
            <Popover>
                <Popover.Trigger
                  class={button({ variant: "ghost", size: "small", square: true, class: "shrink-0" })}
                  onClick={event => event.stopPropagation()}
                >
                    <FiMoreVertical />
                </Popover.Trigger>
                <Popover.Portal>
                    <Popover.Content class="popover-content p-1 w-48 z-50">
                        <menu class="flex flex-col">
                            <button
                              type="button"
                              class="text-left px-3 py-2 rounded-md hover:bg-surface-alt text-sm"
                              onClick={() => navigate({ to: "/acc/$account", params: { account: props.account_identity.toString() } })}
                            >
                                Open
                            </button>
                            <button
                              type="button"
                              class="text-left px-3 py-2 rounded-md hover:bg-surface-alt text-sm"
                              onClick={() => {
                                    setRenameValue(props.account_name);
                                    setRenameOpen(true);
                                }}
                            >
                                Rename
                            </button>
                            <button
                              type="button"
                              class="text-left px-3 py-2 rounded-md hover:bg-surface-alt text-sm"
                              onClick={() => navigate({ to: "/acc/$account/settings", params: { account: props.account_identity.toString() } })}
                            >
                                Settings
                            </button>
                        </menu>
                    </Popover.Content>
                </Popover.Portal>
            </Popover>

            <Modal open={renameOpen()} onOpenChange={setRenameOpen}>
                <Modal.Portal>
                    <Modal.Overlay />
                    <Modal.Positioner>
                        <Modal.Content class="w-full max-w-md bg-surface rounded-md relative mx-auto mt-24">
                            <Modal.CloseButton />
                            <Modal.Title>
                                Rename account
                            </Modal.Title>
                            <div class="p-4 space-y-4">
                                <input
                                  type="text"
                                  class="input w-full"
                                  value={renameValue()}
                                  onInput={event => setRenameValue(event.currentTarget.value)}
                                />
                                <div class="flex justify-end gap-2">
                                    <Modal.CloseButton class={button({ variant: "secondary" })}>
                                        Cancel
                                    </Modal.CloseButton>
                                    <button class={button({ variant: "primary" })} onClick={saveRename}>
                                        Save
                                    </button>
                                </div>
                            </div>
                        </Modal.Content>
                    </Modal.Positioner>
                </Modal.Portal>
            </Modal>
        </>
    );
};
