import { createRootRoute, Outlet } from "@tanstack/solid-router";

import { Navbar } from "#/components/navbar";

export const Route = createRootRoute({
  component: () => (
      <>
        <div class="w-full min-h-screen bg-background h-screen">
          <div class="h-screen flex flex-col">
            <Navbar />
            <div class="flex h-full">
              <Outlet />
            </div>
          </div>
        </div>
      </>
    ),
});
