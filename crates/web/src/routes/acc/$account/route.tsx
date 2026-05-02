import { createFileRoute, Outlet } from "@tanstack/solid-router";
import { Suspense } from "solid-js";

import { Sidebar } from "#/components/sidebar";

export const Route = createFileRoute("/acc/$account")({
  component: () => (
    <div class="w-full h-full grid grid-cols-[auto_1fr] max-h-[calc(100vh-54px)]">
      <div class="h-full">
        <Sidebar />
      </div>
      <div class="w-full h-full overflow-y-auto">
        <Suspense>
          <Outlet />
        </Suspense>
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
