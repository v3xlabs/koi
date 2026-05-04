import { createFileRoute } from "@tanstack/solid-router";
import { For, Show, Suspense } from "solid-js";

import { useQuoters } from "#/api/quoter";
import { QuoterPreview } from "#/components/quoter/preview";

export const Route = createFileRoute("/settings/quoters")({
  component: () => {
    const quotersQuery = useQuoters();

    return (
      <div>
        <div>Quoters</div>
        <Suspense fallback={<div>Loading...</div>}>
          <Show when={quotersQuery.data}>
            {data => (
              <div class="bg-surface rounded-md p-4">
                <ul>
                  <For each={data().quoters}>
                    {quoter => (
                      <div>
                        <QuoterPreview quoter_identity={quoter.quoter_identity} />
                      </div>
                    )}
                  </For>
                </ul>
              </div>
            )}
          </Show>
        </Suspense>
      </div>
    );
  },
});
