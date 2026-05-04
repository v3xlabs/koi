import { Link, useParams } from "@tanstack/solid-router";
import { FaSolidAddressCard, FaSolidClock, FaSolidCopy, FaSolidExternalLink, FaSolidGear, FaSolidGridHorizontal, FaSolidQrcode, FaSolidWallet } from "solid-icons/fa";
import { FiHome } from "solid-icons/fi";
import { For, Show } from "solid-js";

import { useAccount } from "#/api/account";
import { truncateAddress } from "#/utils/address";
import { narrow } from "#/utils/narrow";
import { ReceiveQR } from "#/views/receive/qr";

import { AccountIcon } from "./account/icon";
import { AccountExternalLinkModal } from "./account/link";
import { AccountTypeIcon } from "./account/type";
import { Modal } from "./dialog";

export const Sidebar = () => {
    const params = useParams({ from: "/acc/$account" });
    const account_identity = Number.parseInt(params().account);
    const account = useAccount(() => ({ path: { account_identity } }));

    return (
        <div class="px-1.5 py-2 min-w-56 max-w-64 bg-surface h-full space-y-2">
            <div class="space-y-2 pt-1 w-full">
                <div class="flex items-center gap-2 pl-1 pr-2 py-2 w-full">
                    <div class="size-9 bg-surface-alt rounded-md">
                        <Show when={narrow(() => account.data?.metadata, x => "evm_address" in x)}>
                            {acc => <AccountIcon address={() => acc().evm_address} />}
                        </Show>
                    </div>
                    <div class="leading-none grow">
                        <div class="font-medium text-sm leading-none flex items-center justify-between gap-1">
                            <span>
                                {account.data?.name}
                            </span>
                            <Show when={account.data?.metadata.type}>
                                {type => <AccountTypeIcon type={type} />}
                            </Show>
                        </div>
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
                    <Show when={narrow(() => account.data?.metadata, x => "evm_address" in x)}>
                        {acc => (
                            <ReceiveQR address={() => acc().evm_address}>
                                <Modal.Trigger class="bg-secondary hover:bg-secondary-hover aspect-square rounded-md p-2 flex items-center justify-center cursor-pointer">
                                    <FaSolidQrcode class="w-3.5 h-3.5 text-secondary-foreground" />
                                </Modal.Trigger>
                            </ReceiveQR>
                        )}
                    </Show>
                    <button class="bg-secondary hover:bg-secondary-hover aspect-square rounded-md p-2 flex items-center justify-center cursor-pointer">
                        <FaSolidCopy />
                    </button>
                    <AccountExternalLinkModal account_identity={account_identity} class="bg-secondary hover:bg-secondary-hover aspect-square rounded-md p-2 flex items-center justify-center cursor-pointer">
                        <FaSolidExternalLink />
                    </AccountExternalLinkModal>
                    <For each={[
                        {
                            icon: FaSolidGear,
                            label: "Settings",
                            href: "/acc/$account/settings",
                        },
                    ]}
                    >
                        {item => (
                            <Link
                              to={item.href}
                              class="bg-secondary hover:bg-secondary-hover aspect-square rounded-md p-2 flex items-center justify-center cursor-pointer"
                            >
                                <item.icon class="w-3.5 h-3.5 text-secondary-foreground" />
                            </Link>
                        )}
                    </For>
                </div>
            </div>
            <Show when={account.data?.metadata.type !== "view"}>
                <div>
                    <Link
                      to="/acc/$account/new-tx"
                      params={{ account: params().account }}
                      class="bg-primary hover:bg-primary-hover text-primary-foreground w-full rounded-md p-2 flex items-center gap-2 cursor-pointer justify-center text-sm font-bold"
                    >
                        New transaction
                    </Link>
                </div>
            </Show>
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
                        },
                        {
                            icon: FaSolidGear,
                            label: "Settings",
                            href: "/acc/$account/settings",
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
