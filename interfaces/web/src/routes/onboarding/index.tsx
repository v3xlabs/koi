import { createFileRoute, Link } from "@tanstack/solid-router";

import { button } from "#/components/input/button";

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
        <Link to="/onboarding/networks" class={button({ variant: "primary" })}>
          Let's go
        </Link>
      </div>
    </div>
  ),
});
