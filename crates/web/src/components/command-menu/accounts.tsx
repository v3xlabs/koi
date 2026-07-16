import { useNavigate, useRouterState } from "@tanstack/solid-router";
import { Command } from "cmdk-solid";
import { FaSolidAddressCard } from "solid-icons/fa";
import { FiChevronRight, FiPlus } from "solid-icons/fi";
import { For, Show } from "solid-js";

import { useAccountLayout } from "#/api/account";

import { CommandMenuItem, PagedCommandGroupProperties } from "./item";

const accountFlows = [
    {
        value: "add account",
        keywords: ["create", "import", "wallet"],
        icon: FiPlus,
        title: "Add account",
        description: "Choose how to create or import an account",
        to: "/acc/new",
    },
] as const;

export const AccountCommands = (props: PagedCommandGroupProperties) => {
    const navigate = useNavigate();
    const routerState = useRouterState();
    const layoutQuery = useAccountLayout();
    const accounts = () => layoutQuery.data?.accounts ?? [];
    const showAccountOptions = () => props.page() === "accounts"
      || (props.page() === undefined && props.search().length > 0);

    const go = (to: string) => {
        props.close();
        navigate({ to });
    };

    const switchAccount = (accountId: number) => {
        const currentPath = routerState().location.pathname;
        const nextPath = /^\/acc\/\d+/.test(currentPath)
            ? currentPath.replace(/^\/acc\/\d+/, `/acc/${accountId}`)
            : `/acc/${accountId}`;

        go(nextPath);
    };

    return (
        <>
            <Show when={props.page() === undefined}>
                <Command.Group heading="Accounts">
                    <For each={accountFlows}>
                        {flow => (
                            <CommandMenuItem
                              value={flow.value}
                              keywords={[...flow.keywords]}
                              icon={flow.icon}
                              title={flow.title}
                              description={flow.description}
                              onSelect={() => go(flow.to)}
                            />
                        )}
                    </For>
                    <Show when={accounts().length > 0}>
                        <CommandMenuItem
                          value="switch account"
                          keywords={["wallet", "change account"]}
                          icon={FaSolidAddressCard}
                          title="Switch account..."
                          description={`${accounts().length} account${accounts().length === 1 ? "" : "s"}`}
                          suffix={<FiChevronRight class="size-4 text-muted" />}
                          onSelect={() => props.openPage("accounts")}
                        />
                    </Show>
                </Command.Group>
            </Show>
            <Show when={showAccountOptions()}>
                <Command.Group heading="Switch account">
                    <For each={accounts()}>
                        {account => (
                            <CommandMenuItem
                              value={`switch account ${account.account_identity}`}
                              keywords={[account.name, account.account_identity.toString()]}
                              icon={FaSolidAddressCard}
                              title={account.name}
                              description={`Switch to account #${account.account_identity}`}
                              onSelect={() => switchAccount(account.account_identity)}
                            />
                        )}
                    </For>
                </Command.Group>
            </Show>
        </>
    );
};
