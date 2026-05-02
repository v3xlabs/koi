import { createFileRoute } from "@tanstack/solid-router";
import { For, Show, Suspense } from "solid-js";

import { useQuoters } from "#/api/quoter";
import { QuoterAdd } from "#/components/quoter/add";
import { QuoterPreview } from "#/components/quoter/preview";

export const Route = createFileRoute("/settings/quoters")({
  component: () => {
    const quotersQuery = useQuoters();

    return (
      <div class="w-full space-y-2">
        <div class="flex justify-between items-end">
          <div class="text-lg">Quoters</div>
          <div>
            <QuoterAdd />
          </div>
        </div>
        <Suspense fallback={<div>Loading...</div>}>
          <Show when={quotersQuery.data}>
            {data => (
              <div class="w-full space-y-2 bg-surface rounded-md p-4">
                <Show when={data().quoters.length > 0} fallback={<div class="text-center text-muted">No quoters found</div>}>
                  <ul>
                  <For each={data().quoters}>
                    {quoter => (
                      <div class="py-2 px-4 hover:bg-surface-alt cursor-pointer rounded-md">
                        <QuoterPreview quoter_identity={quoter.quoter_identity} />
                      </div>
                    )}
                  </For>
                  </ul>
                </Show>
              </div>
            )}
          </Show>
        </Suspense>
      </div>
    );
  },
});
