import { Command } from "cmdk-solid";
import { FiArrowLeft, FiCommand } from "solid-icons/fi";
import { createMemo, createSignal, onCleanup, onMount, Show } from "solid-js";

import { AddressExternalLinkModal } from "#/components/link/address";
import { ReceiveQR } from "#/views/receive/qr";

import { AccountCommands } from "./accounts";
import { AssetCommands } from "./assets";
import { ContextualCommands } from "./contextual";
import { CurrentAccountCommands } from "./current-account";
import { DiagnosticCommands } from "./diagnostics";
import { CommandPage } from "./item";
import { NavigationCommands } from "./navigation";
import { PreferenceCommands } from "./preferences";

const createCommandMenuKeyDown = (toggle: () => void) => (event: KeyboardEvent) => {
    if (event.key.toLowerCase() === "k" && (event.metaKey || event.ctrlKey)) {
        event.preventDefault();
        toggle();
    }
};

export const CommandMenu = () => {
    const [open, setOpen] = createSignal(false);
    const [search, setSearch] = createSignal("");
    const [pages, setPages] = createSignal<CommandPage[]>([]);
    const [receiveAddress, setReceiveAddress] = createSignal<string>();
    const [explorerTarget, setExplorerTarget] = createSignal<{ address: string; networks: number[]; }>();
    const page = createMemo(() => pages().at(-1));
    const setMenuOpen = (nextOpen: boolean) => {
        setOpen(nextOpen);

        if (!nextOpen) {
            setSearch("");
            setPages([]);
        }
    };
    const close = () => setMenuOpen(false);
    const openPage = (nextPage: CommandPage) => {
        setSearch("");
        setPages(current => [...current, nextPage]);
    };
    const goBack = () => {
        setSearch("");
        setPages(current => current.slice(0, -1));
    };
    const inputPlaceholder = () => {
        switch (page()) {
            case "accounts": {
                return "Search accounts...";
            }
            case "assets": {
                return "Search tracked assets...";
            }
            case "currency": {
                return "Search display currencies...";
            }
            case "theme": {
                return "Choose a theme...";
            }
            default: {
                return "Search commands, accounts, settings...";
            }
        }
    };
    const handleMenuKeyDown = (event: KeyboardEvent) => {
        if (page() === undefined) return;

        if (event.key === "Escape" || (event.key === "Backspace" && search().length === 0)) {
            event.preventDefault();
            event.stopPropagation();
            goBack();
        }
    };
    const showReceive = (address: string) => {
        close();
        queueMicrotask(() => setReceiveAddress(address));
    };
    const showExplorer = (address: string, networks: number[]) => {
        close();
        queueMicrotask(() => setExplorerTarget({ address, networks }));
    };

    onMount(() => {
        const down = createCommandMenuKeyDown(() => setMenuOpen(!open()));

        document.addEventListener("keydown", down);
        onCleanup(() => document.removeEventListener("keydown", down));
    });

    return (
        <>
            <Command.Dialog
              open={open()}
              onOpenChange={setMenuOpen}
              label="Global command menu"
              loop
              onKeyDown={handleMenuKeyDown}
              overlayClassName="command-menu__overlay"
              contentClassName="command-menu__content"
            >
                <div class="command-menu__input-wrap">
                    <Show
                      when={page() !== undefined}
                      fallback={<FiCommand class="size-4 text-muted" />}
                    >
                        <button
                          type="button"
                          class="flex size-6 items-center justify-center rounded text-muted hover:bg-surface-alt hover:text-foreground"
                          aria-label="Back to all commands"
                          onClick={goBack}
                        >
                            <FiArrowLeft class="size-4" />
                        </button>
                    </Show>
                    <Command.Input
                      value={search()}
                      onValueChange={setSearch}
                      class="command-menu__input"
                      placeholder={inputPlaceholder()}
                    />
                    <kbd class="command-menu__shortcut">Esc</kbd>
                </div>
                <Command.List class="command-menu__list">
                    <Command.Empty class="command-menu__empty">No results found.</Command.Empty>
                    <Show when={page() === undefined}>
                        <ContextualCommands close={close} search={search} />
                        <NavigationCommands close={close} />
                        <CurrentAccountCommands
                          close={close}
                          showReceive={showReceive}
                          showExplorer={showExplorer}
                        />
                        <DiagnosticCommands close={close} />
                    </Show>
                    <AssetCommands
                      close={close}
                      page={page}
                      search={search}
                      openPage={openPage}
                    />
                    <AccountCommands
                      close={close}
                      page={page}
                      search={search}
                      openPage={openPage}
                    />
                    <PreferenceCommands
                      close={close}
                      page={page}
                      search={search}
                      openPage={openPage}
                    />
                </Command.List>
            </Command.Dialog>
            <Show when={receiveAddress()}>
                {address => (
                    <ReceiveQR
                      address={address}
                      open
                      onOpenChange={nextOpen => !nextOpen && setReceiveAddress(undefined)}
                    />
                )}
            </Show>
            <Show when={explorerTarget()}>
                {target => (
                    <AddressExternalLinkModal
                      address={target().address}
                      networks={target().networks}
                      open
                      onOpenChange={nextOpen => !nextOpen && setExplorerTarget(undefined)}
                    />
                )}
            </Show>
        </>
    );
};
