import { createFileRoute } from "@tanstack/solid-router";
import { For, Show, Suspense } from "solid-js";

import { useQuoters } from "#/api/quoter";
import { QuoterAdd } from "#/components/quoter/add";
import { QuoterPreview } from "#/components/quoter/preview";

export const Route = createFileRoute("/settings/quoters")({
  component: () => {
    const quotersQuery = useQuoters();

    return (
      <div class="w-full space-y-4">
        <div class="flex justify-between items-center">
          <div class="text-xl font-bold">Price Feeds</div>
          <QuoterAdd />
        </div>
        <Suspense fallback={<div class="py-8 text-center text-muted">Loading...</div>}>
          <Show when={quotersQuery.data}>
            {data => (
              <Show when={data().quoters.length > 0} fallback={<div class="py-8 text-center text-muted">No quoters found</div>}>
                <ul class="space-y-1">
                  <For each={data().quoters}>
                    {quoter => (
                      <li class="py-2 px-2 hover:bg-surface-alt rounded-lg">
                        <QuoterPreview quoter_identity={quoter.quoter_identity} />
                      </li>
                    )}
                  </For>
                </ul>
              </Show>
            )}
          </Show>
        </Suspense>
      </div>
    );
  },
});
