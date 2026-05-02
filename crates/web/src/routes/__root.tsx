import { createRootRoute, Outlet } from "@tanstack/solid-router";
import { Suspense } from "solid-js";

import { Navbar } from "#/components/navbar";

export const Route = createRootRoute({
  component: () => (
    <>
      <div class="w-full h-screen max-h-screen bg-background flex flex-col justify-stretch">
          <Navbar />
          <div class="h-full w-full">
            <Suspense fallback={<div>Loading...</div>}>
              <Outlet />
            </Suspense>
          </div>
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
