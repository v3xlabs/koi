import { createFileRoute, useSearch } from "@tanstack/solid-router";
import { createMemo } from "solid-js";

import { BuilderTx, TX_PRESETS } from "#/components/tx/builder";
import { TxBuilder } from "#/components/tx/builder/builder";

export type NewTxSearch = {
    type?: BuilderTx["type"];
    asset?: string;
    to?: string;
    amount?: string;
};

const isTransactionType = (value: unknown): value is BuilderTx["type"] => (
  typeof value === "string" && TX_PRESETS.some(preset => preset.type === value)
);

export const Route = createFileRoute("/acc/$account/new-tx")({
  validateSearch: (search: Record<string, unknown>): NewTxSearch => ({
    type: isTransactionType(search.type) ? search.type : undefined,
    asset: typeof search.asset === "string" ? search.asset : undefined,
    to: typeof search.to === "string" ? search.to : undefined,
    amount: typeof search.amount === "string" ? search.amount : undefined,
  }),
  component: () => {
    const search = useSearch({ from: "/acc/$account/new-tx" });
    const initialPrefill = createMemo(() => {
      const type = search().type;

      if (!type) return undefined;

      const data: Record<string, string> = {};
      const { asset, to, amount } = search();

      if (asset !== undefined) data.asset = asset;

      if (to !== undefined) data.to = to;

      if (amount !== undefined) data.amount = amount;

      return { type, data };
    });

    return (
      <div class="w-full space-y-4">
        <div class="">
          <div class="text-lg font-bold">New transaction</div>
          <div class="text-sm text-muted">
            Build transactions step by step
          </div>
        </div>
        <TxBuilder initialPrefill={initialPrefill()} />
      </div>
    );
  },
});
