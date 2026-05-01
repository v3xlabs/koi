import { createFileRoute, Link, Outlet } from "@tanstack/solid-router";
import { For } from "solid-js";

export const Route = createFileRoute("/acc/$account/settings")({
  component: () => (
    <div class="space-y-4">
      <div class="px-4 pt-4">
        Account Settings
        <nav class="bg-surface rounded-md border border-border p-2">
          <ul class="flex gap-2">
            <For each={[
              { label: "Main", href: "/acc/$account/settings" },
              { label: "Tokens", href: "/acc/$account/settings/tokens" },
              { label: "Danger", href: "/acc/$account/settings/danger" },
            ]}
            >
              {item => (
                <li class="text-sm">
                  <Link to={item.href} class="h-full hover:bg-surface-alt rounded-md px-2 py-1" activeProps={{ class: "bg-surface-alt" }} activeOptions={{ exact: true }}>
                    {item.label}
                  </Link>
                </li>
              )}
            </For>
          </ul>
        </nav>
      </div>
      <Outlet />
    </div>
  ),
});
