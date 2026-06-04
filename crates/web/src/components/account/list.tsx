import { makePersisted } from "@solid-primitives/storage";
import { useNavigate } from "@tanstack/solid-router";
import { FaSolidGripVertical } from "solid-icons/fa";
import { FiChevronDown, FiChevronRight, FiPlus } from "solid-icons/fi";
import { Accessor, Component, createMemo, createSignal, For, onCleanup, Show } from "solid-js";

import {
    Account,
    AccountGroup,
    AccountLayout,
    normalizeGroupId,
    useCreateAccountGroup,
    useDeleteAccountGroup,
    useUpdateAccountGroup,
} from "#/api/account";
import { AccountPreview } from "#/components/account/preview";
import { Modal } from "#/components/dialog";
import { button } from "#/components/input/button";

import {
    accountsForGroup,
    applyInsertion,
    DragItem,
    expandGroupForInsertion,
    findInsertion,
    Insertion,
    isDraggingItem,
    matchesInsertionAccount,
    matchesInsertionGroupBefore,
    matchesInsertionGroupEnd,
} from "./layout-dnd";
import { AccountRowMenu } from "./menu";

type AccountsListProps = {
    layout: Accessor<AccountLayout | undefined>;
    editing: Accessor<boolean>;
    draftLayout: Accessor<AccountLayout | undefined>;
    onDraftChange: (layout: AccountLayout) => void;
};

const UNGROUPED_KEY = "ungrouped";

const groupAttr = (group_id?: number) => (
    group_id === undefined ? "ungrouped" : String(group_id)
);

export const AccountsList: Component<AccountsListProps> = (props) => {
    const navigate = useNavigate();
    const [dragItem, setDragItem] = createSignal<DragItem | null>(null);
    const [insertion, setInsertion] = createSignal<Insertion | null>(null);
    const [pointer, setPointer] = createSignal<{ x: number; y: number; } | null>(null);
    const [collapsedGroups, setCollapsedGroups] = makePersisted(
        createSignal<Record<string, boolean>>({}),
        { name: "account-group-collapsed" },
    );
    const [renameGroupId, setRenameGroupId] = createSignal<number | null>(null);
    const [renameGroupName, setRenameGroupName] = createSignal("");
    const [newGroupOpen, setNewGroupOpen] = createSignal(false);
    const [newGroupName, setNewGroupName] = createSignal("");

    const createGroup = useCreateAccountGroup(({ data }) => ({
        contentType: "application/json; charset=utf-8",
        data,
    }));
    const updateGroup = useUpdateAccountGroup(({ data, group_identity }) => ({
        path: { group_identity },
        contentType: "application/json; charset=utf-8",
        data,
    }));
    const deleteGroup = useDeleteAccountGroup(({ group_identity }) => ({
        path: { group_identity },
    }));

    const activeLayout = createMemo(() => (props.editing() ? props.draftLayout() : props.layout()));

    const groups = createMemo(() => (
        activeLayout()?.groups.toSorted((a, b) => a.display_order - b.display_order) ?? []
    ));
    const accounts = createMemo(() => activeLayout()?.accounts ?? []);
    const ungroupedAccounts = createMemo(() => accountsForGroup(accounts(), undefined));
    const hasUngrouped = createMemo(() => ungroupedAccounts().length > 0 || props.editing());
    const showUngroupedHeader = createMemo(() => groups().length > 0);

    const applyDraft = (layout: AccountLayout) => {
        if (props.editing()) {
            props.onDraftChange(layout);
        }
    };

    const expandGroup = (groupId: number | typeof UNGROUPED_KEY) => {
        setCollapsedGroups(groups => ({
            ...groups,
            [groupId.toString()]: false,
        }));
    };

    const endDrag = () => {
        setDragItem(null);
        setInsertion(null);
        setPointer(null);
        document.body.style.removeProperty("user-select");
        document.body.style.removeProperty("cursor");
    };

    onCleanup(endDrag);

    const handlePointerMove = (event: PointerEvent) => {
        const drag = dragItem();

        if (!drag) return;

        setPointer({ x: event.clientX, y: event.clientY });
        setInsertion(findInsertion(event.clientX, event.clientY, drag));
    };

    const startDrag = (item: DragItem) => (event: PointerEvent) => {
        if (!props.editing()) return;

        event.preventDefault();
        event.stopPropagation();

        const handle = event.currentTarget as HTMLElement;

        handle.setPointerCapture(event.pointerId);
        setDragItem(item);
        setPointer({ x: event.clientX, y: event.clientY });
        setInsertion(null);
        document.body.style.userSelect = "none";
        document.body.style.cursor = "grabbing";

        const onMove = (moveEvent: PointerEvent) => handlePointerMove(moveEvent);

        const onEnd = (endEvent: PointerEvent) => {
            handle.releasePointerCapture(endEvent.pointerId);
            window.removeEventListener("pointermove", onMove);
            window.removeEventListener("pointerup", onEnd);
            window.removeEventListener("pointercancel", onEnd);

            const drag = dragItem();
            const layout = activeLayout();
            const point = pointer();
            const target = insertion()
                ?? (drag && point ? findInsertion(point.x, point.y, drag) : null);

            if (drag && target && layout) {
                const next = applyInsertion(layout, drag, target);

                applyDraft(next);

                const expand = expandGroupForInsertion(target);

                if (expand !== null) {
                    expandGroup(expand);
                }
            }

            endDrag();
        };

        window.addEventListener("pointermove", onMove);
        window.addEventListener("pointerup", onEnd);
        window.addEventListener("pointercancel", onEnd);
    };

    const toggleGroup = (groupId: number | typeof UNGROUPED_KEY) => {
        if (dragItem()) return;

        setCollapsedGroups(groups => ({
            ...groups,
            [groupId.toString()]: !groups[groupId.toString()],
        }));
    };

    const isCollapsed = (groupId: number | typeof UNGROUPED_KEY) => collapsedGroups[groupId.toString()] ?? false;

    const saveRenameGroup = () => {
        const groupId = renameGroupId();

        if (groupId === null) return;

        if (props.editing()) {
            const layout = activeLayout();

            if (layout) {
                props.onDraftChange({
                    ...layout,
                    groups: layout.groups.map(group => (
                        group.group_identity === groupId
                            ? { ...group, name: renameGroupName() }
                            : group
                    )),
                });
            }

            setRenameGroupId(null);

            return;
        }

        updateGroup.mutate({
            group_identity: groupId,
            data: { name: renameGroupName() },
        }, {
            onSuccess: () => setRenameGroupId(null),
        });
    };

    const createNewGroup = () => {
        createGroup.mutate({ data: { name: newGroupName() } }, {
            onSuccess: (group) => {
                setNewGroupOpen(false);
                setNewGroupName("");

                if (props.editing()) {
                    const layout = activeLayout();

                    if (!layout) return;

                    props.onDraftChange({
                        ...layout,
                        groups: [...layout.groups, group],
                    });
                }
            },
        });
    };

    const removeGroup = (group_identity: number) => {
        if (props.editing()) {
            const layout = activeLayout();

            if (!layout) return;

            props.onDraftChange({
                ...layout,
                groups: layout.groups.filter(group => group.group_identity !== group_identity),
                accounts: layout.accounts.map(account => (
                    normalizeGroupId(account.group_id) === group_identity
                        ? { ...account, group_id: undefined }
                        : account
                )),
            });

            return;
        }

        deleteGroup.mutate({ group_identity });
    };

    const InsertionLine: Component = () => (
        <div class="absolute inset-x-2 h-0.5 rounded-full bg-primary pointer-events-none z-20" />
    );

    const AccountRow: Component<{ account: Account; group_id?: number; }> = rowProps => {
        const dragging = () => isDraggingItem(
            dragItem(),
            { type: "account", account_identity: rowProps.account.account_identity, group_id: rowProps.group_id },
        );
        const showBefore = () => matchesInsertionAccount(
            insertion(),
            rowProps.account.account_identity,
            rowProps.group_id,
            "before",
        );
        const showAfter = () => matchesInsertionAccount(
            insertion(),
            rowProps.account.account_identity,
            rowProps.group_id,
            "after",
        );

        return (
            <div
              data-drop-account={rowProps.account.account_identity}
              data-drop-group={groupAttr(rowProps.group_id)}
              class="relative flex items-center gap-2 px-2 hover:bg-surface-alt rounded-lg py-2 min-w-0"
              classList={{
                    "opacity-40 pointer-events-none": dragging(),
                }}
              onClick={() => {
                    if (!props.editing() && !dragItem()) {
                        navigate({ to: "/acc/$account", params: { account: rowProps.account.account_identity.toString() } });
                    }
                }}
            >
                <Show when={showBefore()}>
                    <div class="absolute inset-x-2 top-0 -translate-y-1/2">
                        <InsertionLine />
                    </div>
                </Show>
                <Show when={showAfter()}>
                    <div class="absolute inset-x-2 bottom-0 translate-y-1/2">
                        <InsertionLine />
                    </div>
                </Show>
                <Show when={props.editing()}>
                    <button
                      type="button"
                      class="touch-none shrink-0 rounded p-0.5 text-muted hover:text-foreground cursor-grab active:cursor-grabbing"
                      onPointerDown={startDrag({
                            type: "account",
                            account_identity: rowProps.account.account_identity,
                            group_id: rowProps.group_id,
                        })}
                    >
                        <FaSolidGripVertical class="size-4" />
                    </button>
                </Show>
                <div class="min-w-0 flex-1 cursor-pointer">
                    <AccountPreview account_identity={rowProps.account.account_identity} />
                </div>
                <div onClick={event => event.stopPropagation()}>
                    <AccountRowMenu
                      account_identity={rowProps.account.account_identity}
                      account_name={rowProps.account.name}
                    />
                </div>
            </div>
        );
    };

    const GroupHeader: Component<{ group: AccountGroup; }> = headerProps => {
        const dragging = () => isDraggingItem(
            dragItem(),
            { type: "group", group_identity: headerProps.group.group_identity },
        );
        const targeted = () => matchesInsertionGroupBefore(insertion(), headerProps.group.group_identity)
            || (dragItem()?.type === "account" && matchesInsertionGroupEnd(insertion(), headerProps.group.group_identity));

        return (
            <div
              data-drop-group-header={headerProps.group.group_identity}
              class="relative flex items-center gap-2 px-2 py-2 text-sm font-semibold text-muted rounded-lg"
              classList={{
                    "opacity-40 pointer-events-none": dragging(),
                    "bg-primary/10 ring-1 ring-primary/50 text-foreground": targeted() && !dragging(),
                }}
            >
                <Show when={props.editing()}>
                    <button
                      type="button"
                      class="touch-none shrink-0 rounded p-0.5 text-muted hover:text-foreground cursor-grab active:cursor-grabbing"
                      onPointerDown={startDrag({ type: "group", group_identity: headerProps.group.group_identity })}
                    >
                        <FaSolidGripVertical class="size-4" />
                    </button>
                </Show>
                <button
                  type="button"
                  class="flex items-center gap-1 hover:text-foreground"
                  onClick={() => toggleGroup(headerProps.group.group_identity)}
                >
                    <Show when={isCollapsed(headerProps.group.group_identity)} fallback={<FiChevronDown class="size-4" />}>
                        <FiChevronRight class="size-4" />
                    </Show>
                    {headerProps.group.name}
                </button>
                <Show when={props.editing()}>
                    <div class="ml-auto flex items-center gap-1">
                        <button
                          type="button"
                          class={button({ variant: "ghost", size: "small" })}
                          onClick={() => {
                                setRenameGroupId(headerProps.group.group_identity);
                                setRenameGroupName(headerProps.group.name);
                            }}
                        >
                            Rename
                        </button>
                        <button
                          type="button"
                          class={button({ variant: "ghost", size: "small" })}
                          onClick={() => removeGroup(headerProps.group.group_identity)}
                        >
                            Delete
                        </button>
                    </div>
                </Show>
            </div>
        );
    };

    const draggedAccount = createMemo(() => {
        const drag = dragItem();

        if (!drag || drag.type !== "account") return null;

        return activeLayout()?.accounts.find(account => account.account_identity === drag.account_identity);
    });

    return (
        <div class="space-y-1">
            <For each={groups()}>
                {group => (
                    <section class="space-y-1">
                        <GroupHeader group={group} />
                        <div
                          data-drop-group-zone={group.group_identity}
                          class="rounded-lg"
                          classList={{
                                "min-h-8": props.editing(),
                                "border border-dashed border-primary/40 bg-primary/5": props.editing()
                                    && matchesInsertionGroupEnd(insertion(), group.group_identity),
                            }}
                        >
                            <Show when={!isCollapsed(group.group_identity)}>
                                <For each={accountsForGroup(accounts(), group.group_identity)}>
                                    {account => <AccountRow account={account} group_id={group.group_identity} />}
                                </For>
                            </Show>
                        </div>
                    </section>
                )}
            </For>

            <Show when={hasUngrouped()}>
                <section class="space-y-1">
                    <Show when={showUngroupedHeader()}>
                        <div
                          data-drop-group-header="ungrouped"
                          class="relative flex items-center gap-2 rounded-lg px-2 py-2 text-sm font-semibold text-muted"
                          classList={{
                                "bg-primary/10 ring-1 ring-primary/50 text-foreground": props.editing()
                                    && dragItem()?.type === "account"
                                    && matchesInsertionGroupEnd(insertion(), undefined),
                            }}
                        >
                            <Show when={props.editing()}>
                                <span class="size-4" />
                            </Show>
                            <button
                              type="button"
                              class="flex items-center gap-1 hover:text-foreground"
                              onClick={() => toggleGroup(UNGROUPED_KEY)}
                            >
                                <Show when={isCollapsed(UNGROUPED_KEY)} fallback={<FiChevronDown class="size-4" />}>
                                    <FiChevronRight class="size-4" />
                                </Show>
                                Ungrouped
                            </button>
                        </div>
                    </Show>
                    <div
                      data-drop-group-zone="ungrouped"
                      class="rounded-lg"
                      classList={{
                            "min-h-8": props.editing(),
                            "border border-dashed border-primary/40 bg-primary/5": props.editing()
                                && matchesInsertionGroupEnd(insertion(), undefined),
                        }}
                    >
                        <Show when={!isCollapsed(UNGROUPED_KEY) || !showUngroupedHeader()}>
                            <For each={ungroupedAccounts()}>
                                {account => <AccountRow account={account} />}
                            </For>
                        </Show>
                    </div>
                </section>
            </Show>

            <Show when={props.editing()}>
                <button
                  type="button"
                  class={button({ variant: "outline", class: "w-full mt-2 text-sm" })}
                  onClick={() => setNewGroupOpen(true)}
                >
                    <FiPlus class="size-4" />
                    New group
                </button>
            </Show>

            <Show when={dragItem()?.type === "account" && draggedAccount() && pointer()}>
                <div
                  class="fixed z-50 pointer-events-none w-[calc(100%-2rem)] max-w-md rounded-lg border border-primary/30 bg-surface px-3 py-2 shadow-lg"
                  style={{
                        left: `${Math.min(pointer()!.x + 12, window.innerWidth - 320)}px`,
                        top: `${pointer()!.y + 12}px`,
                    }}
                >
                    <AccountPreview account_identity={draggedAccount()!.account_identity} />
                </div>
            </Show>

            <Modal open={renameGroupId() !== null} onOpenChange={open => !open && setRenameGroupId(null)}>
                <Modal.Portal>
                    <Modal.Overlay />
                    <Modal.Positioner>
                        <Modal.Content class="w-full max-w-md bg-surface rounded-md relative mx-auto mt-24">
                            <Modal.CloseButton />
                            <Modal.Title>Rename group</Modal.Title>
                            <div class="p-4 space-y-4">
                                <input
                                  type="text"
                                  class="input w-full"
                                  value={renameGroupName()}
                                  onInput={event => setRenameGroupName(event.currentTarget.value)}
                                />
                                <div class="flex justify-end gap-2">
                                    <Modal.CloseButton class={button({ variant: "secondary" })}>Cancel</Modal.CloseButton>
                                    <button class={button({ variant: "primary" })} onClick={saveRenameGroup}>Save</button>
                                </div>
                            </div>
                        </Modal.Content>
                    </Modal.Positioner>
                </Modal.Portal>
            </Modal>

            <Modal open={newGroupOpen()} onOpenChange={setNewGroupOpen}>
                <Modal.Portal>
                    <Modal.Overlay />
                    <Modal.Positioner>
                        <Modal.Content class="w-full max-w-md bg-surface rounded-md relative mx-auto mt-24">
                            <Modal.CloseButton />
                            <Modal.Title>New group</Modal.Title>
                            <div class="p-4 space-y-4">
                                <input
                                  type="text"
                                  class="input w-full"
                                  placeholder="Group name"
                                  value={newGroupName()}
                                  onInput={event => setNewGroupName(event.currentTarget.value)}
                                />
                                <div class="flex justify-end gap-2">
                                    <Modal.CloseButton class={button({ variant: "secondary" })}>Cancel</Modal.CloseButton>
                                    <button class={button({ variant: "primary" })} onClick={createNewGroup}>Create</button>
                                </div>
                            </div>
                        </Modal.Content>
                    </Modal.Positioner>
                </Modal.Portal>
            </Modal>
        </div>
    );
};

export const cloneLayout = (layout: AccountLayout): AccountLayout => ({
    groups: layout.groups.map(group => ({ ...group })),
    accounts: layout.accounts.map(account => ({ ...account })),
});
