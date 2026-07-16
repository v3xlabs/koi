import { Toast, toaster } from "@kobalte/core/toast";
import { useNavigate, useRouterState } from "@tanstack/solid-router";
import { Command } from "cmdk-solid";
import { FaSolidAddressCard, FaSolidGear, FaSolidRefresh } from "solid-icons/fa";
import { FiCopy, FiFilePlus } from "solid-icons/fi";
import { createMemo, For, Show } from "solid-js";

import { refreshAccountBalances, useAccount } from "#/api/account";
import { useDisplayCurrency } from "#/api/context";
import { TX_PRESETS } from "#/components/tx/builder";
import { narrow } from "#/utils/narrow";

import { CommandGroupProperties, CommandMenuItem } from "./item";

type AccountRoute = "" | "assets" | "history" | "settings" | "settings/assets" | "new-tx";

const showToast = (message: string) => toaster.show(props => (
    <Toast toastId={props.toastId} class="toast">
        <div>{message}</div>
    </Toast>
));

export const CurrentAccountCommands = (props: CommandGroupProperties) => {
    const navigate = useNavigate();
    const routerState = useRouterState();
    const { displayCurrency } = useDisplayCurrency();
    const currentAccountId = createMemo(() => {
        const match = routerState().location.pathname.match(/^\/acc\/(\d+)/);

        return match?.[1] ? Number.parseInt(match[1]) : undefined;
    });
    const currentAccount = useAccount(() => ({
        path: { account_identity: currentAccountId() ?? 0 },
    }), { enabled: () => currentAccountId() !== undefined });
    const currentAddress = createMemo(
        () => narrow(() => currentAccount.data?.metadata, metadata => "evm_address" in metadata)?.evm_address,
    );
    const canSign = createMemo(
        () => currentAccount.data !== undefined && currentAccount.data.metadata.type !== "view",
    );

    const goToAccountRoute = (route: AccountRoute, search?: Record<string, string>) => {
        const accountId = currentAccountId();

        if (accountId === undefined) return;

        const suffix = route ? `/${route}` : "";

        props.close();
        navigate({
            to: `/acc/${accountId}${suffix}`,
            search,
        });
    };

    const copyCurrentAddress = () => {
        const address = currentAddress();

        if (!address) return;

        props.close();
        void navigator.clipboard.writeText(address).then(
            () => showToast("Address copied"),
            () => showToast("Failed to copy address"),
        );
    };

    const refreshBalances = () => {
        const accountId = currentAccountId();

        if (accountId === undefined) return;

        props.close();
        void refreshAccountBalances({
            path: { account_identity: accountId },
            query: { display_currency: displayCurrency() },
        }).then(
            () => showToast("Balances refreshed"),
            () => showToast("Failed to refresh balances"),
        );
    };

    return (
        <Show when={currentAccountId() !== undefined}>
            <Command.Group heading="Current account">
                <CommandMenuItem
                  value="current account dashboard"
                  keywords={["home", "balances", "pending transactions"]}
                  icon={FaSolidAddressCard}
                  title="Open account dashboard"
                  description="View balances and pending transactions"
                  onSelect={() => goToAccountRoute("")}
                />
                <Show when={canSign()}>
                    <For each={TX_PRESETS}>
                        {preset => (
                            <CommandMenuItem
                              value={`current account transaction ${preset.type}`}
                              keywords={["build", "transaction", "new transaction", preset.name]}
                              icon={preset.icon}
                              title={preset.name}
                              description={preset.description}
                              onSelect={() => goToAccountRoute("new-tx", { type: preset.type })}
                            />
                        )}
                    </For>
                </Show>
                <CommandMenuItem
                  value="current account assets"
                  keywords={["tokens", "balances"]}
                  icon={FiFilePlus}
                  title="Account assets"
                  description="View this account's tracked assets"
                  onSelect={() => goToAccountRoute("assets")}
                />
                <CommandMenuItem
                  value="current account manage assets"
                  keywords={["tokens", "enable", "disable"]}
                  icon={FiFilePlus}
                  title="Manage account assets"
                  description="Choose which assets this account tracks"
                  onSelect={() => goToAccountRoute("settings/assets")}
                />
                <CommandMenuItem
                  value="current account history"
                  keywords={["transactions", "activity"]}
                  icon={FaSolidRefresh}
                  title="Transaction history"
                  description="Open this account's activity"
                  onSelect={() => goToAccountRoute("history")}
                />
                <CommandMenuItem
                  value="refresh current account balances"
                  keywords={["reload", "sync", "update"]}
                  icon={FaSolidRefresh}
                  title="Refresh balances"
                  description="Fetch fresh balances for this account"
                  onSelect={refreshBalances}
                />
                <CommandMenuItem
                  value="current account settings"
                  keywords={["rename", "configure"]}
                  icon={FaSolidGear}
                  title="Account settings"
                  description="Rename or configure this account"
                  onSelect={() => goToAccountRoute("settings")}
                />
                <Show when={currentAddress()}>
                    <CommandMenuItem
                      value="copy current account address"
                      keywords={["clipboard", "evm", "receive"]}
                      icon={FiCopy}
                      title="Copy account address"
                      description="Copy the current EVM address"
                      onSelect={copyCurrentAddress}
                    />
                </Show>
            </Command.Group>
        </Show>
    );
};
