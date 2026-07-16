import { useNavigate, useRouterState } from "@tanstack/solid-router";
import { Command } from "cmdk-solid";
import { FaSolidAddressCard } from "solid-icons/fa";
import { FiPlus } from "solid-icons/fi";
import { For } from "solid-js";

import { useAccountLayout } from "#/api/account";

import { CommandGroupProperties, CommandMenuItem } from "./item";

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

export const AccountCommands = (props: CommandGroupProperties) => {
    const navigate = useNavigate();
    const routerState = useRouterState();
    const layoutQuery = useAccountLayout();

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
            <For each={layoutQuery.data?.accounts}>
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
    );
};
