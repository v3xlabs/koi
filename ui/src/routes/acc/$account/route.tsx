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
  notFoundComponent: () => (
    <div class="w-full px-4">
      <div class="bg-surface p-4 rounded-md mx-auto mt-4">
        Not found
      </div>
    </div>
  ),
});
