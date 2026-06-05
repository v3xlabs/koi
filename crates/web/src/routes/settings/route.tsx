import { createFileRoute, Link, Outlet } from "@tanstack/solid-router";
import { FaSolidCalculator, FaSolidCoins, FaSolidHandshake, FaSolidNetworkWired } from "solid-icons/fa";
import { For } from "solid-js";

import { Navbar } from "#/components/navbar";

const navItems = [
  { label: "Main", href: "/settings", icon: null },
  { label: "Networks", href: "/settings/networks", icon: FaSolidNetworkWired },
  { label: "Assets", href: "/settings/assets", icon: FaSolidCoins },
  { label: "Price Feeds", href: "/settings/quoters", icon: FaSolidCalculator },
  { label: "Vendors", href: "/settings/vendors", icon: FaSolidHandshake },
] as const;

export const Route = createFileRoute("/settings")({
  component: () => (
    <>
      <Navbar />
      <div class="w-full p-4 pb-64">
        <div class="mx-auto w-full max-w-3xl mt-8">
          <div class="bg-surface py-5 px-4 rounded-xl w-full">
            <div class="flex gap-6 min-w-0">
              <nav class="shrink-0 w-36 border-r border-border pr-4">
                <ul class="space-y-1">
                  <For each={navItems}>
                    {item => (
                      <li>
                        <Link
                          to={item.href}
                          class="w-full px-2 py-2 rounded-lg hover:bg-surface-alt flex items-center gap-2 text-sm font-bold data-[status=active]:bg-surface-alt"
                          activeOptions={{ exact: true }}
                        >
                          {item.icon && item.icon?.({ class: "w-3.5 h-3.5 shrink-0" })}
                          {item.label}
                        </Link>
                      </li>
                    )}
                  </For>
                </ul>
              </nav>
              <div class="flex-1 min-w-0">
                <Outlet />
              </div>
            </div>
          </div>
        </div>
      </div>
    </>
  ),
});
