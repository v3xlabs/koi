import { createRootRoute, Outlet } from "@tanstack/solid-router";
import { Suspense } from "solid-js";

import { CommandMenu } from "#/components/command-menu";

export const Route = createRootRoute({
  component: () => (
    <>
      <div class="w-full h-screen max-h-screen min-w-0 bg-background">
        <CommandMenu />
        <Suspense fallback={<div>Loading...</div>}>
          <Outlet />
        </Suspense>
      </div>
    </>
  ),
  notFoundComponent: () => (
    <div class="w-full px-4">
      <div class="bg-surface p-4 rounded-md mx-auto mt-6">
        Not found
      </div>
    </div>
  ),
});
