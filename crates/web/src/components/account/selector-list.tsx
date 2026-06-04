import { FiChevronDown } from "solid-icons/fi";
import { Component, createEffect, createMemo, For, on, Show } from "solid-js";

import { Account, AccountLayout } from "#/api/account";

import { accountsForGroup } from "./layout-dnd";
import { AccountSwitcherItem } from "./switcher-item";

type AccountSelectorListProps = {
    layout: AccountLayout | undefined;
    activeAccountId?: number;
    open?: boolean;
    onSelect: (account_identity: number) => void;
};

const SelectorRow: Component<{
    account: Account;
    active: boolean;
    onSelect: () => void;
    setActiveRef?: (el: HTMLButtonElement) => void;
}> = (props) => (
    <button
      type="button"
      ref={el => {
            if (props.active && el && props.setActiveRef) {
                props.setActiveRef(el);
            }
        }}
      class="w-full cursor-pointer text-left transition-colors"
      classList={{
            "rounded-md bg-surface px-3 py-2 hover:bg-surface-alt": props.active,
            "rounded-md px-2 py-2 hover:bg-surface-alt": !props.active,
        }}
      onClick={props.onSelect}
    >
        <div class="flex items-center gap-2">
            <div class="min-w-0 flex-1">
                <AccountSwitcherItem account_identity={props.account.account_identity} />
            </div>
            <FiChevronDown
              class="size-4 shrink-0 text-muted"
              classList={{ invisible: !props.active }}
            />
        </div>
    </button>
);

export const AccountSelectorList: Component<AccountSelectorListProps> = (props) => {
    let scrollContainer: HTMLDivElement | undefined;
    let activeRow: HTMLButtonElement | undefined;

    const groups = createMemo(() => (
        props.layout?.groups.toSorted((a, b) => a.display_order - b.display_order) ?? []
    ));
    const accounts = createMemo(() => props.layout?.accounts ?? []);
    const ungroupedAccounts = createMemo(() => accountsForGroup(accounts(), undefined));
    const hasUngrouped = createMemo(() => ungroupedAccounts().length > 0);
    const showUngroupedHeader = createMemo(() => groups().length > 0);

    const alignActiveRow = () => {
        if (!scrollContainer || !activeRow) return;

        const containerTop = scrollContainer.getBoundingClientRect().top;
        const rowTop = activeRow.getBoundingClientRect().top;

        scrollContainer.scrollTop += rowTop - containerTop;
    };

    createEffect(on(
        () => [props.open, props.activeAccountId, props.layout] as const,
        ([open]) => {
            if (!open) return;

            requestAnimationFrame(() => {
                alignActiveRow();
                requestAnimationFrame(alignActiveRow);
            });
        },
    ));

    return (
        <Show
          when={accounts().length > 0}
          fallback={<div class="px-2 py-4 text-center text-sm text-muted">No accounts yet.</div>}
        >
            <div
              ref={scrollContainer}
              class="max-h-[min(20rem,60vh)] w-full min-w-0 overflow-x-hidden overflow-y-auto"
            >
                <For each={groups()}>
                    {group => (
                        <section>
                            <div class="px-3 py-1.5 text-[11px] font-semibold uppercase tracking-wide text-muted">
                                {group.name}
                            </div>
                            <For each={accountsForGroup(accounts(), group.group_identity)}>
                                {account => (
                                    <SelectorRow
                                      account={account}
                                      active={account.account_identity === props.activeAccountId}
                                      setActiveRef={el => { activeRow = el; }}
                                      onSelect={() => props.onSelect(account.account_identity)}
                                    />
                                )}
                            </For>
                        </section>
                    )}
                </For>

                <Show when={hasUngrouped()}>
                    <section>
                        <Show when={showUngroupedHeader()}>
                            <div class="px-3 py-1.5 text-[11px] font-semibold uppercase tracking-wide text-muted">
                                Ungrouped
                            </div>
                        </Show>
                        <For each={ungroupedAccounts()}>
                            {account => (
                                <SelectorRow
                                  account={account}
                                  active={account.account_identity === props.activeAccountId}
                                  setActiveRef={el => { activeRow = el; }}
                                  onSelect={() => props.onSelect(account.account_identity)}
                                />
                            )}
                        </For>
                    </section>
                </Show>
            </div>
        </Show>
    );
};
