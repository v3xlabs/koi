import { createFileRoute, Link, Outlet } from "@tanstack/solid-router";
import { For } from "solid-js";

import { button } from "#/components/input/button";

export const Route = createFileRoute("/acc/$account/settings")({
  component: () => (
    <div class="space-y-2">
      <nav class="bg-surface rounded-md p-2">
        <ul class="flex gap-2">
          <For each={[
            { label: "Main", href: "/acc/$account/settings" },
            { label: "Assets", href: "/acc/$account/settings/assets" },
            { label: "Danger", href: "/acc/$account/settings/danger" },
          ]}
          >
            {item => (
              <li class="text-sm">
                <Link
                  to={item.href}
                  class={button({ variant: "ghost", size: "small", class: "h-full" })}
                  activeProps={{ class: "bg-surface-alt" }}
                  activeOptions={{ exact: true }}
                >
                  {item.label}
                </Link>
              </li>
            )}
          </For>
        </ul>
      </nav>
      <Outlet />
    </div>
  ),
});
