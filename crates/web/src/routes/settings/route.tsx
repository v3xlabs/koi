import { createFileRoute, Link, Outlet } from "@tanstack/solid-router";
import { FaSolidCalculator, FaSolidCoins, FaSolidHandshake, FaSolidNetworkWired } from "solid-icons/fa";
import { For } from "solid-js";

export const Route = createFileRoute("/settings")({
  component: () => (
    <div class="mx-auto pt-8 space-y-4 grid grid-cols-[200px_1fr] gap-4 w-fit h-fit pb-64">
      <nav class="bg-surface rounded-md p-2 h-fit">
        <ul>
          <For each={[
            {
              label: "Main",
              href: "/settings",
            },
            {
              label: "Networks",
              href: "/settings/networks",
              icon: FaSolidNetworkWired,
            },
            {
              label: "Assets",
              href: "/settings/assets",
              icon: FaSolidCoins,
            },
            {
              label: "Price Feeds",
              href: "/settings/quoters",
              icon: FaSolidCalculator,
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
      </nav>
      <div class="w-full max-w-2xl min-w-0 space-y-4">
        <Outlet />
      </div>
    </div>
  ),
});
