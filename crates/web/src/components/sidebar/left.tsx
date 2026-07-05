import { Link } from "@tanstack/solid-router";
import { FaSolidAddressCard, FaSolidGear } from "solid-icons/fa";
import { FiMenu } from "solid-icons/fi";
import { createSignal, For, onCleanup, onMount } from "solid-js";

import { button, cn } from "../input/button";
import { Branding } from "../navbar/branding";

const navItems = [
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
] as const;

const [expanded, setExpanded] = createSignal(false);
const [mountedCount, setMountedCount] = createSignal(0);

export const sidebarExpanded = expanded;
export const sidebarExists = () => mountedCount() > 0;
export const sidebarBrandingActive = () => sidebarExists() && sidebarExpanded();

export const SidebarLeft = () => {
    const widthClass = () => (expanded() ? "w-52" : "w-14");

    onMount(() => {
        setMountedCount(count => count + 1);
        onCleanup(() => setMountedCount(count => Math.max(0, count - 1)));
    });

    return (
        <div class={cn("h-full shrink-0 transition-[width] duration-300 ease-[cubic-bezier(0.22,1,0.36,1)]", widthClass())}>
            <div class={cn("h-full bg-surface fixed top-0 bottom-0 z-20 overflow-hidden transition-[width] duration-300 ease-[cubic-bezier(0.22,1,0.36,1)]", widthClass())}>
                <div class="flex h-full flex-col">
                    <div class="flex h-14 shrink-0 items-center gap-1 px-2">
                        <button
                          type="button"
                          class={button({ square: true, size: "large", variant: "ghost" })}
                          aria-label={expanded() ? "Collapse sidebar" : "Expand sidebar"}
                          aria-expanded={expanded()}
                          onClick={() => setExpanded(value => !value)}
                        >
                            <FiMenu />
                        </button>
                        <div
                          class={cn(
                                "min-w-0 overflow-hidden transition-[width,opacity,transform] duration-300 ease-[cubic-bezier(0.22,1,0.36,1)]",
                                expanded() ? "w-36 translate-x-0 opacity-100 delay-75" : "w-0 -translate-x-2 opacity-0",
                            )}
                          aria-hidden={!expanded()}
                        >
                            <Branding class="p-0" tabIndex={expanded() ? undefined : -1} />
                        </div>
                    </div>
                    <div class="grow" />
                    <nav class="flex flex-col gap-1 p-2">
                        <For each={navItems}>
                            {item => (
                                <Link
                                  to={item.href}
                                  class={cn(
                                        button({ variant: "ghost", size: "large", square: !expanded() }),
                                        expanded() && "w-full justify-start overflow-hidden px-3 text-sm font-bold",
                                    )}
                                  title={item.label}
                                >
                                    {item.icon({ class: "shrink-0" })}
                                    <span
                                      class={cn(
                                            "min-w-0 truncate transition-[opacity,transform] duration-300 ease-[cubic-bezier(0.22,1,0.36,1)]",
                                            expanded() ? "translate-x-0 opacity-100 delay-75" : "hidden -translate-x-2 opacity-0",
                                        )}
                                    >
                                        {item.label}
                                    </span>
                                </Link>
                            )}
                        </For>
                    </nav>
                </div>
            </div>
        </div>
    );
};
