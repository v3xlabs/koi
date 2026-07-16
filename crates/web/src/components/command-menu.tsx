import { Toast, toaster } from "@kobalte/core/toast";
import { useNavigate, useRouterState } from "@tanstack/solid-router";
import { Command } from "cmdk-solid";
import { FaSolidAddressCard, FaSolidArrowRight, FaSolidGear, FaSolidRefresh } from "solid-icons/fa";
import { FiCommand, FiCopy, FiEye, FiEyeOff, FiFilePlus, FiGlobe, FiLock, FiMoon, FiPlus, FiSun } from "solid-icons/fi";
import { Component, createMemo, createSignal, For, onCleanup, onMount, Setter, Show, useContext } from "solid-js";

import { appcontext } from "#/api";
import { useAccount, useAccountLayout } from "#/api/account";
import { narrow } from "#/utils/narrow";

const createCommandMenuKeyDown = (setOpen: Setter<boolean>) => (event: KeyboardEvent) => {
    if (event.key.toLowerCase() === "k" && (event.metaKey || event.ctrlKey)) {
        event.preventDefault();
        setOpen(value => !value);
    }
};

const CommandMenuItem: Component<{
    value: string;
    keywords?: string[];
    icon: Component<{ class?: string; }>;
    title: string;
    description?: string;
    onSelect: () => void;
}> = props => (
    <Command.Item
      value={props.value}
      keywords={props.keywords}
      onSelect={props.onSelect}
      class="command-menu__item"
    >
        <div class="command-menu__item-icon">
            <props.icon class="size-4" />
        </div>
        <div class="min-w-0 flex-1">
            <div class="truncate text-sm font-medium">{props.title}</div>
            <Show when={props.description}>
                {description => <div class="truncate text-xs text-muted">{description()}</div>}
            </Show>
        </div>
    </Command.Item>
);

export const CommandMenu = () => {
    const navigate = useNavigate();
    const routerState = useRouterState();
    const { privacyMode: [privacyMode, setPrivacyMode], theme: [theme, setTheme] } = useContext(appcontext);
    const layoutQuery = useAccountLayout();
    const [open, setOpen] = createSignal(false);

    const currentAccountId = createMemo(() => {
        const match = routerState().location.pathname.match(/^\/acc\/(\d+)/);

        return match?.[1] ? Number.parseInt(match[1]) : undefined;
    });

    const currentAccount = useAccount(() => ({
      path: { account_identity: currentAccountId() ?? 0 },
    }), { enabled: () => currentAccountId() !== undefined });

    const run = (action: () => void) => {
        action();
        setOpen(false);
    };

    const go = (to: string) => run(() => navigate({ to }));

    const goToAccountRoute = (route: "" | "assets" | "history" | "settings" | "new-tx", search?: Record<string, string>) => {
        const accountId = currentAccountId();

        if (!accountId) return;

        const suffix = route ? `/${route}` : "";

        run(() => navigate({
            to: `/acc/${accountId}${suffix}`,
            search,
        }));
    };

    const switchAccount = (accountId: number) => {
        run(() => {
            const currentPath = routerState().location.pathname;
            const nextPath = /^\/acc\/\d+/.test(currentPath)
                ? currentPath.replace(/^\/acc\/\d+/, `/acc/${accountId}`)
                : `/acc/${accountId}`;

            navigate({ to: nextPath });
        });
    };

    const copyCurrentAddress = () => {
        const address = narrow(() => currentAccount.data?.metadata, x => "evm_address" in x)?.evm_address;

        if (!address) return;

        run(() => {
            void navigator.clipboard.writeText(address).then(
                () => toaster.show(props => (
                    <Toast toastId={props.toastId} class="toast">
                        <div>Address copied</div>
                    </Toast>
                )),
                () => toaster.show(props => (
                    <Toast toastId={props.toastId} class="toast">
                        <div>Failed to copy address</div>
                    </Toast>
                )),
            );
        });
    };

    onMount(() => {
        const down = createCommandMenuKeyDown(setOpen);

        document.addEventListener("keydown", down);
        onCleanup(() => document.removeEventListener("keydown", down));
    });

    return (
        <Command.Dialog
          open={open()}
          onOpenChange={setOpen}
          label="Global command menu"
          loop
          overlayClassName="command-menu__overlay"
          contentClassName="command-menu__content"
        >
            <div class="command-menu__input-wrap">
                <FiCommand class="size-4 text-muted" />
                <Command.Input class="command-menu__input" placeholder="Search commands, accounts, settings..." />
                <kbd class="command-menu__shortcut">Esc</kbd>
            </div>
            <Command.List class="command-menu__list">
                <Command.Empty class="command-menu__empty">No results found.</Command.Empty>

                <Command.Group heading="Navigate">
                    <CommandMenuItem
                      value="go accounts"
                      keywords={["home", "wallets"]}
                      icon={FaSolidAddressCard}
                      title="Accounts"
                      description="Open the account list"
                      onSelect={() => go("/")}
                    />
                    <CommandMenuItem
                      value="go address book"
                      keywords={["contacts"]}
                      icon={FaSolidAddressCard}
                      title="Address book"
                      description="Open saved contacts"
                      onSelect={() => go("/addressbook")}
                    />
                    <CommandMenuItem
                      value="go settings"
                      keywords={["preferences"]}
                      icon={FaSolidGear}
                      title="Settings"
                      description="Open app settings"
                      onSelect={() => go("/settings")}
                    />
                    <CommandMenuItem
                      value="go networks"
                      keywords={["chains", "rpc"]}
                      icon={FiGlobe}
                      title="Networks"
                      description="Manage chains and endpoints"
                      onSelect={() => go("/settings/networks")}
                    />
                    <CommandMenuItem
                      value="go assets"
                      keywords={["tokens"]}
                      icon={FiFilePlus}
                      title="Assets"
                      description="Manage known assets"
                      onSelect={() => go("/settings/assets")}
                    />
                </Command.Group>

                <Show when={currentAccountId()}>
                    <Command.Group heading="Current account">
                        <CommandMenuItem
                          value="current account dashboard"
                          icon={FaSolidAddressCard}
                          title="Open account dashboard"
                          description="View balances and pending transactions"
                          onSelect={() => goToAccountRoute("")}
                        />
                        <CommandMenuItem
                          value="current account send"
                          keywords={["transfer", "transaction"]}
                          icon={FaSolidArrowRight}
                          title="Send"
                          description="Build a send transaction"
                          onSelect={() => goToAccountRoute("new-tx", { type: "send" })}
                        />
                        <CommandMenuItem
                          value="current account assets"
                          keywords={["tokens", "balances"]}
                          icon={FiFilePlus}
                          title="Account assets"
                          description="Manage assets for this account"
                          onSelect={() => goToAccountRoute("assets")}
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
                          value="current account settings"
                          icon={FaSolidGear}
                          title="Account settings"
                          description="Rename or configure this account"
                          onSelect={() => goToAccountRoute("settings")}
                        />
                        <Show when={narrow(() => currentAccount.data?.metadata, x => "evm_address" in x)}>
                            <CommandMenuItem
                              value="copy current account address"
                              keywords={["clipboard", "evm"]}
                              icon={FiCopy}
                              title="Copy account address"
                              description="Copy the current EVM address"
                              onSelect={copyCurrentAddress}
                            />
                        </Show>
                    </Command.Group>
                </Show>

                <Command.Group heading="Accounts">
                    <CommandMenuItem
                      value="add account"
                      keywords={["create", "import", "wallet"]}
                      icon={FiPlus}
                      title="Add account"
                      description="Create or import a wallet account"
                      onSelect={() => go("/acc/new")}
                    />
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

                <Command.Group heading="Preferences">
                    <CommandMenuItem
                      value="toggle privacy mode"
                      keywords={["hide balances", "private"]}
                      icon={privacyMode() ? FiEye : FiEyeOff}
                      title={privacyMode() ? "Disable privacy mode" : "Enable privacy mode"}
                      description="Hide or reveal sensitive wallet values"
                      onSelect={() => run(() => setPrivacyMode(!privacyMode()))}
                    />
                    <CommandMenuItem
                      value="set light theme"
                      keywords={["appearance"]}
                      icon={FiSun}
                      title="Use light theme"
                      description="Switch the interface to light mode"
                      onSelect={() => run(() => setTheme("light"))}
                    />
                    <CommandMenuItem
                      value="set dark theme"
                      keywords={["appearance"]}
                      icon={FiMoon}
                      title="Use dark theme"
                      description="Switch the interface to dark mode"
                      onSelect={() => run(() => setTheme("dark"))}
                    />
                    <CommandMenuItem
                      value="set system theme"
                      keywords={["appearance", "auto"]}
                      icon={FiLock}
                      title="Use system theme"
                      description={`Current theme: ${theme()}`}
                      onSelect={() => run(() => setTheme("system"))}
                    />
                </Command.Group>
            </Command.List>
        </Command.Dialog>
    );
};
