import { createFileRoute, useParams } from "@tanstack/solid-router";
import { FaSolidArrowRight, FaSolidRefresh } from "solid-icons/fa";
import { Show, Suspense, useContext } from "solid-js";

import { appcontext } from "#/api";
import { useAccount, useAccountBalances } from "#/api/account";
import { AccountAssetSummary } from "#/components/account/asset/summary";
import { AssetAmount } from "#/components/asset/amount";
import { Modal } from "#/components/dialog";
import { button } from "#/components/input/button";
import { narrow } from "#/utils/narrow";
import { ReceiveQR } from "#/views/receive/qr";

export const Route = createFileRoute("/acc/$account/")({
  component: () => {
    const params = useParams({ from: "/acc/$account" });
    const { displayCurrency: [displayCurrency] } = useContext(appcontext);
    // const balance = useAccountBalance(params().account);
    const account_identity = Number.parseInt(params().account);
    const balanceQuery = useAccountBalances(() => ({ path: { account_identity }, query: { display_currency: displayCurrency() } }));
    const account = useAccount(() => ({ path: { account_identity } }));

    return (
      <div class="p-4 space-y-4">
        <div class="grid grid-cols-1 xl:grid-cols-5 w-full gap-4">
          <div class="space-y-4 w-full xl:col-span-3">
            <div class="bg-surface p-4 rounded-md space-y-4">
              <div class="flex justify-between items-center">
                <div class="text-sm font-bold text-muted">
                  Total balance
                </div>
                <div class="text-muted text-sm flex items-center gap-2">
                  <Suspense>
                    <span>
                      Updated
                      {" "}
                      {balanceQuery.data?.updated_at ? Date.parse(balanceQuery.data.updated_at).toLocaleString() : "-"}
                    </span>
                  </Suspense>
                  <Show when={!balanceQuery.isLoading}>
                    <button
                      class={button({ variant: "ghost", size: "small", square: true })}
                      classList={{
                        "animate-spin": balanceQuery.isRefetching,
                      }}
                      onClick={() => balanceQuery.refetch()}
                    >
                      <FaSolidRefresh class="w-3.5 h-3.5" />
                    </button>
                  </Show>
                </div>
              </div>
              <div class="flex justify-between items-center">
                <div class="text-4xl font-bold tabular-nums">
                  <Suspense fallback={<span class="text-muted">Loading...</span>}>
                    <Show when={balanceQuery.data}>
                      {data => (
                        <AssetAmount amount={() => BigInt(data().total_quote ?? "0")} asset={() => data().asset} />
                      )}
                    </Show>
                  </Suspense>
                </div>
                <div class="flex gap-2">
                  <Show when={account.data?.metadata.type !== "view"}>
                    <button
                      class={button({ variant: "primary", class: "w-full text-sm font-bold" })}
                    >
                      <FaSolidArrowRight class="-rotate-45" />
                      Send
                    </button>
                  </Show>
                  <Show when={narrow(() => account.data?.metadata, x => "evm_address" in x)}>
                    {x => (
                      <ReceiveQR address={() => x().evm_address}>
                        <Modal.Trigger
                          class={button({ variant: "secondary", class: "w-full text-sm font-bold" })}
                        >
                          <FaSolidArrowRight class="-rotate-225" />
                          Receive
                        </Modal.Trigger>
                      </ReceiveQR>
                    )}
                  </Show>
                </div>
              </div>
            </div>
            <div class="w-full">
              <AccountAssetSummary account_identity={account_identity} />
            </div>
          </div>
          <div class="bg-surface p-4 xl:col-span-2 rounded-md max-h-128">
            <div>
              <div>
                Pending transactions
              </div>
            </div>
          </div>
        </div>
      </div>
    );
  },
});
