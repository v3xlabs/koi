import { Link } from "@tanstack/solid-router";
import { FiLock, FiUnlock } from "solid-icons/fi";

import { usePrivacyMode } from "#/api/context";

import logo from "../../assets/kohaku.svg";
import { ConnectionButton } from "../connection";
import { NetworkWidget } from "./networks";

export const Navbar = () => {
    const { privacyMode, setPrivacyMode } = usePrivacyMode();

    return (
        <div class="border-b border-background px-2 flex items-stretch justify-between bg-surface w-full min-w-0 shrink-0">
            <div class="flex min-w-0 items-center gap-2 py-2">
                <Link to="/" class="flex min-w-0 items-center gap-2">
                    <div class="w-8 h-8 shrink-0">
                        <img src={logo} alt="Koi" class="w-full h-full object-contain" />
                    </div>
                    <div class="leading-none min-w-0">
                        <h1 class="font-bold">Koi</h1>
                        <span class="text-muted text-sm whitespace-nowrap">a privacy wallet</span>
                    </div>
                </Link>
            </div>
            <div class="min-w-0">
            </div>
            <div class="flex shrink-0 items-stretch self-stretch">
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
