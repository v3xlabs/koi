import { Link } from "@tanstack/solid-router";
import { FaSolidAddressCard, FaSolidGear } from "solid-icons/fa";
import { FiMenu } from "solid-icons/fi";
import { For } from "solid-js";

import { button } from "../input/button";

export const SidebarLeft = () => (
    <div class="w-14 h-full">
        <div class="w-14 h-full bg-surface flex flex-col justify-between fixed top-0 bottom-0">
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
    </div>
);
