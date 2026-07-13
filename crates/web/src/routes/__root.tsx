import { createRootRoute, Outlet } from "@tanstack/solid-router";
import { Suspense } from "solid-js";

import { PendingWalletRequests } from "#/components/wallet-request/pending";

export const Route = createRootRoute({
  component: () => (
    <>
      <div class="w-full h-screen max-h-screen min-w-0 bg-background">
        <Suspense fallback={<div>Loading...</div>}>
          <Outlet />
        </Suspense>
      </div>
      <PendingWalletRequests />
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
