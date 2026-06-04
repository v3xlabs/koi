import { FiLock, FiUnlock } from "solid-icons/fi";

import { usePrivacyMode } from "#/api/context";

import { ConnectionButton } from "../connection";
import { AccountNavbarActions, AccountSwitcher } from "./account-switcher";
import { NetworkWidget } from "./networks";

export const Navbar = () => {
    const { privacyMode, setPrivacyMode } = usePrivacyMode();

    return (
        <div class="mt-1 flex w-full min-w-0 shrink-0 items-stretch justify-between px-4 py-2">
            <div class="flex min-w-0 items-center gap-2">
                <AccountSwitcher />
                <AccountNavbarActions />
            </div>
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
