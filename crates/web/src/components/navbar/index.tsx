import { useLocation } from "@tanstack/solid-router";
import { FiLock, FiUnlock } from "solid-icons/fi";
import { createMemo, Show } from "solid-js";

import { usePrivacyMode } from "#/api/context";

import { ConnectionButton } from "../connection";
import { AccountNavbarActions, AccountSwitcher } from "./account-switcher";
import { Branding } from "./branding";
import { NetworkWidget } from "./networks";

export const Navbar = () => {
    const { privacyMode, setPrivacyMode } = usePrivacyMode();
    const x = useLocation();
    const isAccountPage = createMemo(() => x().href.startsWith("/acc/"));

    return (
        <div class="mt-1 flex w-full min-w-0 shrink-0 items-stretch justify-between px-4 py-2">
            <Show
              when={isAccountPage()}
              fallback={(
                    <Branding />
                )}
            >
                <div class="flex min-w-0 items-center gap-2">
                    <AccountSwitcher />
                    <AccountNavbarActions />
                </div>
            </Show>
            <div class="min-w-0" />
            <div class="flex shrink-0 items-stretch gap-2 self-stretch">
                <ConnectionButton />
                <NetworkWidget />
                <button
                  class="nav-icon-button"
                  type="button"
                  aria-label={privacyMode() ? "Disable privacy mode" : "Enable privacy mode"}
                  aria-pressed={privacyMode()}
                  title={privacyMode() ? "Disable privacy mode" : "Enable privacy mode"}
                  onClick={() => setPrivacyMode(!privacyMode())}
                >
                    {privacyMode() ? <FiLock /> : <FiUnlock />}
                </button>
            </div>
        </div>
    );
};
