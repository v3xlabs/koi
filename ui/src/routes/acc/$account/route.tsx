import { createFileRoute, Outlet } from "@tanstack/solid-router";

import { Sidebar } from "#/components/sidebar";

export const Route = createFileRoute("/acc/$account")({
  component: () => (
    <div class="flex h-full w-full">
      <Sidebar />
      <div class="overflow-y-auto w-full">
        <Outlet />
      </div>
    </div>
  ),
});
