import { createFileRoute, Outlet } from "@tanstack/solid-router";

import { Navbar } from "#/components/navbar";

export const Route = createFileRoute("/acc/new")({
  component: () => (
    <>
      <Navbar />
      <Outlet />
    </>
  ),
});
