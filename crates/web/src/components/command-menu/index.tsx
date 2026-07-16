import { Command } from "cmdk-solid";
import { FiCommand } from "solid-icons/fi";
import { createSignal, onCleanup, onMount, Setter } from "solid-js";

import { AccountCommands } from "./accounts";
import { CurrentAccountCommands } from "./current-account";
import { NavigationCommands } from "./navigation";
import { PreferenceCommands } from "./preferences";

const createCommandMenuKeyDown = (setOpen: Setter<boolean>) => (event: KeyboardEvent) => {
    if (event.key.toLowerCase() === "k" && (event.metaKey || event.ctrlKey)) {
        event.preventDefault();
        setOpen(value => !value);
    }
};

export const CommandMenu = () => {
    const [open, setOpen] = createSignal(false);
    const close = () => setOpen(false);

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
                <NavigationCommands close={close} />
                <CurrentAccountCommands close={close} />
                <AccountCommands close={close} />
                <PreferenceCommands close={close} />
            </Command.List>
        </Command.Dialog>
    );
};
