import { createFileRoute, Outlet } from "@tanstack/solid-router";

import { Navbar } from "#/components/navbar";

export const Route = createFileRoute("/acc/import")({
  component: () => (
    <>
      <Navbar />
      <Outlet />
    </>
  ),
});
