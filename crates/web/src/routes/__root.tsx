import { createRootRoute, Outlet } from "@tanstack/solid-router";
import { Suspense } from "solid-js";

export const Route = createRootRoute({
  component: () => (
    <>
      <div class="w-full h-screen max-h-screen min-w-0 bg-background flex flex-col justify-stretch">
          <div class="flex-1 min-h-0 min-w-0 w-full overflow-x-clip">
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
