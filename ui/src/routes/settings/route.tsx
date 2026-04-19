import { createFileRoute, Link, Outlet } from "@tanstack/solid-router";
import { FaSolidCalculator, FaSolidCoins, FaSolidHandshake, FaSolidNetworkWired } from "solid-icons/fa";
import { For } from "solid-js";

export const Route = createFileRoute("/settings")({
  component: () => (
    <div class="p-4 w-full flex justify-center">
      <div class="flex gap-4 w-full">
        <div class="bg-surface rounded-md w-full max-w-xs h-fit">
          <ul>
            <For each={[
              {
                label: "Main",
                href: "/settings",
              },
              {
                label: "Tokens",
                href: "/settings/tokens",
                icon: FaSolidCoins,
              },
              {
                label: "Quoters",
                href: "/settings/quoters",
                icon: FaSolidCalculator,
              },
              {
                label: "Networks",
                href: "/settings/networks",
                icon: FaSolidNetworkWired,
              },
              {
                label: "Vendors",
                href: "/settings/vendors",
                icon: FaSolidHandshake,
              },
            ]}
            >
              {item => (
                <li class="w-full">
                  <Link
                    to={item.href}
                    class="w-full p-2 rounded-md hover:bg-surface-alt flex items-center gap-2 data-[status=active]:bg-surface-alt"
                    activeOptions={{
                      exact: true,
                    }}
                  >
                    {item.icon?.({ class: "w-3.5 h-3.5" })}
                    {item.label}
                  </Link>
                </li>
              )}
            </For>
          </ul>
        </div>
        <Outlet />
      </div>
    </div>
  ),
});
