import { createFileRoute, Link, Outlet } from "@tanstack/solid-router";
import { For } from "solid-js";

export const Route = createFileRoute("/onboarding")({
  component: () => (
    <div class="mx-auto w-full max-w-2xl pt-8 space-y-4">
      <nav class="bg-surface rounded-md border border-border p-2">
        <ul class="flex gap-2">
          <For each={[
            { label: "Main", href: "/onboarding" },
            { label: "Networks", href: "/onboarding/networks" },
            { label: "Vendors", href: "/onboarding/vendors" },
            { label: "Accounts", href: "/onboarding/accounts" },
          ]}
          >
            {item => (
              <li class="text-sm">
                <Link to={item.href} class="h-full hover:bg-surface-alt rounded-md px-2 py-1" activeProps={{ class: "bg-surface-alt" }}>
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
