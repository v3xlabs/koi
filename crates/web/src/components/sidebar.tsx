import { Link, useParams } from "@tanstack/solid-router";
import { FaSolidAddressCard, FaSolidClock, FaSolidGear, FaSolidGridHorizontal, FaSolidWallet } from "solid-icons/fa";
import { FiHome, FiMenu, FiPlus } from "solid-icons/fi";
import { Component, For, JSXElement, Show } from "solid-js";

import { useAccount } from "#/api/account";

import { button } from "./input/button";
import { AccountNavbarActions, AccountSwitcher } from "./navbar/account-switcher";
import { Branding } from "./navbar/branding";

const NavLink: Component<{ href: string; icon?: Component; children: JSXElement; }> = ({ href, icon, children }) => (
    <Link
      to={href}
      class="hover:bg-surface-alt w-full rounded-md px-4 py-2 text-sm font-bold flex items-center gap-4 cursor-pointer data-[status=active]:bg-surface-alt group"
      activeOptions={{
            exact: true,
        }}
    >
        {icon && icon({ class: "w-3.5 h-3.5 group-data-[status=active]:text-primary" })}
        {children}
    </Link>
);

export const Sidebar = () => {
    const params = useParams({ from: "/acc/$account" });
    const account_identity = Number.parseInt(params().account);
    const account = useAccount(() => ({ path: { account_identity } }));

    return (
        <div class="flex h-full">
            <div class="w-screen max-w-14 h-full bg-surface flex flex-col justify-between">
                <div class="p-2 flex flex-col justify-center items-center">
                    <button class={button({ square: true, size: "large", variant: "ghost" })}>
                        <FiMenu />
                    </button>
                </div>
                <div class="flex flex-col justify-center items-center p-2">
                    <For each={[
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
                    ]}
                    >
                        {item => (
                            <Link to={item.href} class={button({ variant: "ghost", size: "large", square: true })} title={item.label}>
                                {item.icon({})}
                            </Link>
                        )}
                    </For>
                </div>
            </div>
            <div class="px-4 py-2 w-screen max-w-64 h-full space-y-2 flex flex-col">
                <Branding />
                <div class="space-y-2">
                    <AccountSwitcher />
                </div>
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
                <div class="divide-y divide-border flex flex-col grow">
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
                    ]}
                    >
                        {group => (
                            <div class="py-2 first:pt-0 space-y-[3px]">
                                <For each={group}>
                                    {item => (
                                        <NavLink href={item.href} icon={item.icon}>
                                            {item.label}
                                        </NavLink>
                                    )}
                                </For>
                            </div>
                        )}
                    </For>
                </div>
                <div>
                    ... last synced
                </div>
            </div>
        </div>
    );
};
