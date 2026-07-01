import { createFileRoute } from "@tanstack/solid-router";

import { Navbar } from "#/components/navbar";
import { SidebarLeft } from "#/components/sidebar/left";

export const Route = createFileRoute("/addressbook")({
  component: () => (
    <div class="w-full flex h-full">
      <SidebarLeft />
      <div class="grow">
        <Navbar />
        <div class="w-full pb-64 px-4 space-y-4">
          <div>
            <div class="text-2xl font-bold">
              Addressbook
            </div>
            <div class="text-sm text-muted">
              Manage address you interact with, assign labels, tags, and more.
            </div>
          </div>
          <div class="bg-surface p-4 rounded-md">
            This page is still under development.
          </div>
        </div>
      </div>
    </div>
  ),
});
