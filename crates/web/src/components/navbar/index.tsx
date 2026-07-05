import { useMatch } from "@tanstack/solid-router";
import { FiEye, FiEyeOff } from "solid-icons/fi";
import { Show } from "solid-js";

import { usePrivacyMode } from "#/api/context";

import { ConnectionButton } from "../connection";
import { sidebarBrandingActive } from "../sidebar/left";
import { AccountNavbarActions } from "./account-switcher";
import { Branding } from "./branding";
import { NetworkWidget } from "./networks";

export const Navbar = () => {
    const { privacyMode, setPrivacyMode } = usePrivacyMode();
    const onAccount = useMatch({ from: "/acc/$account", shouldThrow: false });

    return (
        <div class="mt-1 flex w-full min-w-0 shrink-0 items-stretch justify-between pr-4 py-2">
            <Show
              when={onAccount()}
              fallback={(
                <Show when={!sidebarBrandingActive()}>
                    <div class="pl-4">
                        <Branding />
                    </div>
                </Show>
                )}
            >
                <div class="flex min-w-0 items-center gap-4 pl-4">
                    <input type="text" class="input" placeholder="Search" />
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
                    {privacyMode() ? <FiEyeOff /> : <FiEye />}
                </button>
            </div>
        </div>
    );
};
