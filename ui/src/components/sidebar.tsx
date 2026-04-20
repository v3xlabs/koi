import { Link, useParams } from "@tanstack/solid-router";
import { FaSolidAddressCard, FaSolidClock, FaSolidCopy, FaSolidExternalLink, FaSolidGear, FaSolidGridHorizontal, FaSolidQrcode, FaSolidWallet } from "solid-icons/fa";
import { FiHome } from "solid-icons/fi";
import { For, Show } from "solid-js";

import { useAccount } from "#/api/account";
import { truncateAddress } from "#/utils/address";
import { narrow } from "#/utils/narrow";
import { ReceiveQR } from "#/views/receive/qr";

import { AccountIcon } from "./account/icon";
import { Modal } from "./dialog";

export const Sidebar = () => {
    const params = useParams({ from: "/acc/$account" });
    const account = useAccount(params().account);

    return (
        <div class="border-r px-1.5 py-2 min-w-56 max-w-64 bg-surface border-r-border h-full space-y-2">
            <div class="space-y-2 pt-1">
                <div class="flex items-center gap-2 pl-1 py-2">
                    <div class="size-9 bg-surface-alt rounded-md">
                        <Show when={narrow(() => account.data?.metadata, x => "evm_address" in x)}>
                            {acc => <AccountIcon address={() => acc().evm_address} />}
                        </Show>
                    </div>
                    <div class="leading-none">
                        <div class="font-medium text-sm leading-none">{account.data?.name}</div>
                        <Show when={narrow(() => account.data?.metadata, x => "evm_address" in x)}>
                            {acc => (
                                <div class="text-muted text-sm leading-none">
                                    {truncateAddress(acc().evm_address)}
                                </div>
                            )}
                        </Show>
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
                <Link
                    to="/acc/$account/new-tx"
                    class="bg-primary hover:bg-primary-hover text-primary-foreground w-full rounded-md p-2 flex items-center gap-2 cursor-pointer justify-center text-sm font-bold"
                >
                    New transaction
                </Link>
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
                            icon: FaSolidClock,
                            label: "History",
                            href: "/acc/$account/history",
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
                        {
                            icon: FaSolidGear,
                            label: "Settings",
                            href: "/settings",
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
};
