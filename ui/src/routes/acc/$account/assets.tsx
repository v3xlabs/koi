import { createFileRoute } from "@tanstack/solid-router";
import { For } from "solid-js";

export const Route = createFileRoute("/acc/$account/assets")({
  component: () => (
    <div class="w-full p-4">
      <div class="w-full max-w-3xl space-y-4">
        <div class="flex items-end justify-between">
          <div class="text-xl">
            Assets
          </div>
        </div>
        <div class="bg-surface px-5 py-2.5 rounded-md w-full border border-border">
          <div class="border-b border-border pb-2.5 py-1 text-sm uppercase">
            Assets
          </div>
          <div class="divide-y divide-border">
            <For each={[
              {
                name: "ETH",
              },
              {
                name: "USDC",
              },
              {
                name: "USDT",
              },
              {
                name: "DAI",
              },
              {
                name: "WBTC",
              },
            ]}
            >
              {asset => (
                <div classList={{
                  "flex items-center gap-2 py-3.5 relative z-10": true,
                  "before:opacity-0 hover:before:opacity-100 before:transition-all before:-z-10 before:absolute before:inset-y-0 before:inset-x-0 hover:before:-inset-x-2.5 before:bg-surface-alt before:rounded-md": true,
                }}
                >
                  <div class="size-10 bg-surface-alt border border-border rounded-full">

                  </div>
                  <div>
                    {asset.name}
                  </div>
                </div>
              )}
            </For>
          </div>
        </div>
      </div>
    </div>
  ),
});
