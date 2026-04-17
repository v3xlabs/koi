import { createFileRoute } from "@tanstack/solid-router";

export const Route = createFileRoute("/")({
  component: () => {
    // something
    Route.useParams();

    return <div>Hello "/"!</div>;
  },
});
