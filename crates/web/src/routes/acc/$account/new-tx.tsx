import { createFileRoute, useSearch } from "@tanstack/solid-router";

import { TxBuilder } from "#/components/tx/builder/builder";

export type NewTxSearch = {
    type?: string;
    asset?: string;
    to?: string;
    amount?: string;
};

export const Route = createFileRoute("/acc/$account/new-tx")({
  validateSearch: (search: Record<string, unknown>): NewTxSearch => ({
    type: typeof search.type === "string" ? search.type : undefined,
    asset: typeof search.asset === "string" ? search.asset : undefined,
    to: typeof search.to === "string" ? search.to : undefined,
    amount: typeof search.amount === "string" ? search.amount : undefined,
  }),
  component: () => {
    const search = useSearch({ from: "/acc/$account/new-tx" });

    return (
      <div class="w-full space-y-4">
        <div class="">
          <div class="text-lg font-bold">New transaction</div>
          <div class="text-sm text-muted">
            Build transactions step by step
          </div>
        </div>
        <TxBuilder initialPrefill={search().type ? { type: "send" as const, data: { asset: search().asset, to: search().to, amount: search().amount } } : undefined} />
      </div>
    );
  },
});
