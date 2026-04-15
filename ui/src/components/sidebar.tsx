import { For } from "solid-js";
import { FiHome } from "solid-icons/fi";
import { FaSolidArrows, FaSolidCopy, FaSolidExternalLink, FaSolidGear, FaSolidGridHorizontal, FaSolidQrcode, FaSolidWallet } from "solid-icons/fa";

export const Sidebar = () => {
    // Sidebar comment

    return (
        <div class="border-r px-1.5 py-2 min-w-56 max-w-64 bg-surface border-r-border h-full space-y-2">
            <div class="space-y-2">
                <div class="flex items-center gap-2 pl-1">
                    <div class="w-8 h-8 bg-surface-alt rounded-md">

                    </div>
                    <div class="leading-none">
                        <div class="font-medium text-sm leading-none">Wallet Name</div>
                        <div class="text-muted text-sm leading-none">
                            0x123...4567
                        </div>
                    </div>
                </div>
                <div class="flex gap-2 px-1">
                    <For each={[
                        {
                            icon: FaSolidQrcode,
                            label: "Receive",
                        },
                        {
                            icon: FaSolidCopy,
                            label: "Copy address",
                        },
                        {
                            icon: FaSolidExternalLink,
                            label: "View on Explorer",
                        },
                        {
                            icon: FaSolidGear,
                            label: "Settings",
                        }
                    ]}>
                        {(item) => (
                            <button class="bg-surface-alt aspect-square rounded-md p-2 flex items-center justify-center cursor-pointer">
                                <item.icon class="w-3.5 h-3.5" />
                            </button>
                        )}
                    </For>
                </div>
            </div>
            <div>
                <button class="bg-primary hover:bg-primary-hover text-primary-foreground w-full rounded-md p-2 flex items-center gap-2 cursor-pointer justify-center text-sm font-bold">
                    New transaction
                </button>
            </div>
            <div class="space-y-0">
                <For each={[
                    {
                        icon: FiHome,
                        label: "Home",
                        href: "/",
                    },
                    {
                        icon: FaSolidWallet,
                        label: "Assets",
                        href: "/assets",
                    },
                    {
                        icon: FaSolidArrows,
                        label: "Swap",
                        href: "/swap"
                    },
                    {
                        icon: FaSolidArrows,
                        label: "Earn",
                        href: "/earn"
                    },
                    {
                        icon: FaSolidGridHorizontal,
                        label: "Apps",
                    }
                ]}>
                    {(item) => (
                        <button class="hover:bg-surface-alt w-full rounded-md p-2 flex items-center gap-2 cursor-pointer">
                            <item.icon />
                            {item.label}
                        </button>
                    )}
                </For>
            </div>
        </div>
    )
};
