import { Link, useParams } from "@tanstack/solid-router";
import { FaSolidAddressCard, FaSolidClock, FaSolidGear, FaSolidGridHorizontal, FaSolidWallet } from "solid-icons/fa";
import { FiHome, FiPlus } from "solid-icons/fi";
import { For, Show } from "solid-js";

import { useAccount } from "#/api/account";

import logo from "../assets/kohaku.svg";
import { button } from "./input/button";

export const Sidebar = () => {
    const params = useParams({ from: "/acc/$account" });
    const account_identity = Number.parseInt(params().account);
    const account = useAccount(() => ({ path: { account_identity } }));

    return (
        <div class="px-1.5 py-2 min-w-56 max-w-64 bg-surface h-full space-y-2">
            <Link to="/" class="flex min-w-0 items-center gap-2 p-2">
                <div class="w-8 h-8 shrink-0">
                    <img src={logo} alt="Koi" class="w-full h-full object-contain" />
                </div>
                <div class="leading-none min-w-0">
                    <h1 class="font-bold">Koi</h1>
                    <span class="text-muted text-sm whitespace-nowrap">just a wallet</span>
                </div>
            </Link>
            <Show when={account.data && account.data?.metadata.type !== "view"}>
                <div>
                    <Link
                      to="/acc/$account/new-tx"
                      params={{ account: params().account }}
                      class={button({ variant: "primary", class: "w-full text-sm font-bold flex items-center gap-1" })}
                    >
                        <FiPlus />
                        New transaction
                    </Link>
                </div>
            </Show>
            <div class="divide-y divide-border flex flex-col justify-between">
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
                                      class="hover:bg-surface-alt w-full rounded-md px-4 py-2 text-sm font-bold flex items-center gap-4 cursor-pointer data-[status=active]:bg-surface-alt group"
                                      activeOptions={{
                                            exact: true,
                                        }}
                                    >
                                        <item.icon class="w-3.5 h-3.5 group-data-[status=active]:text-primary" />
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
