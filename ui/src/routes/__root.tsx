import { createRootRoute, Outlet } from "@tanstack/solid-router";
import { Suspense } from "solid-js";

import { Navbar } from "#/components/navbar";

export const Route = createRootRoute({
  component: () => (
    <>
      <div class="w-full min-h-screen bg-background h-screen">
        <div class="h-screen flex flex-col">
          <Navbar />
          <div class="flex h-[calc(100vh - 56px)] overflow-y-auto">
            <Suspense fallback={<div>Loading...</div>}>
              <Outlet />
            </Suspense>
          </div>
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
