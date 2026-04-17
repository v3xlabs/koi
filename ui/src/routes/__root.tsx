import { Navbar } from "#/components/navbar";
import { Sidebar } from "#/components/sidebar";
import { createRootRoute, Outlet } from "@tanstack/solid-router";

import { Show, Suspense } from "solid-js";

export const Route = createRootRoute({
  component: RootComponent,
});

function RootComponent() {
  console.log("RootComponent");

  return (
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
  );
}
