import { createFileRoute, Link } from "@tanstack/solid-router";

export const Route = createFileRoute("/onboarding/")({
  component: () => (
    <div class="w-full p-4 bg-surface rounded-md border border-border">
      <div>
        Welcome to Koi,
      </div>
      <div>
        Before we can get started we will need to configure a few things.
      </div>
      <div class="flex justify-end">
        <Link to="/onboarding/networks" class="btn btn-primary">
          Let's go
        </Link>
      </div>
    </div>
  ),
});
