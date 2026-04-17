import { createFileRoute } from "@tanstack/solid-router";

export const Route = createFileRoute("/acc/$account/assets")({
  component: () => <div>Hello "/acc/$account/assets"!</div>,
});
