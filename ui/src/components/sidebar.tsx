import { Link } from "@tanstack/solid-router";
import { FaSolidAddressCard, FaSolidArrows, FaSolidCopy, FaSolidExternalLink, FaSolidGear, FaSolidGridHorizontal, FaSolidQrcode, FaSolidWallet } from "solid-icons/fa";
import { FiHome } from "solid-icons/fi";
import { For } from "solid-js";

import { ReceiveQR } from "#/views/receive/qr";

import { Modal } from "./dialog";

export const Sidebar = () => (
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
                <ReceiveQR address={() => "0x1234567890123456789012345678901234567890"}>
                    <Modal.Trigger class="bg-secondary hover:bg-secondary-hover aspect-square rounded-md p-2 flex items-center justify-center cursor-pointer">
                        <FaSolidQrcode class="w-3.5 h-3.5 text-secondary-foreground" />
                    </Modal.Trigger>
                </ReceiveQR>
                <For each={[
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
                    },
                ]}
                >
                    {item => (
                        <button class="bg-secondary hover:bg-secondary-hover aspect-square rounded-md p-2 flex items-center justify-center cursor-pointer">
                            <item.icon class="w-3.5 h-3.5 text-secondary-foreground" />
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
        <div class="divide-y divide-border">
            <For each={[
                [
                    {
                        icon: FiHome,
                        label: "Home",
                        href: "/acc/$account/",
                    },
                    {
                        icon: FaSolidWallet,
                        label: "Assets",
                        href: "/acc/$account/assets",
                    },
                    {
                        icon: FaSolidArrows,
                        label: "Swap",
                        href: "/acc/$account/swap",
                    },
                    {
                        icon: FaSolidArrows,
                        label: "Earn",
                        href: "/acc/$account/earn",
                    },
                    {
                        icon: FaSolidGridHorizontal,
                        label: "Apps",
                        href: "/acc/$account/apps",
                    }],
                [
                    {
                        icon: FaSolidAddressCard,
                        label: "Addressbook",
                        href: "/addressbook",
                    },
                ],
            ]}
            >
                {group => (
                    <div class="py-2 first:pt-0">
                        <For each={group}>
                            {item => (
                                <Link
                                  to={item.href}
                                  class="hover:bg-surface-alt w-full rounded-md px-4 py-2 text-sm font-bold flex items-center gap-4 cursor-pointer data-[status=active]:bg-surface-alt"
                                  activeOptions={{
                                        exact: true,
                                    }}
                                >
                                    <item.icon class="w-3.5 h-3.5" />
                                    {item.label}
                                </Link>
                            )}
                        </For>
                    </div>
                )}
            </For>
        </div>
    </div>
);
