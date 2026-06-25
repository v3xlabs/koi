import { createFileRoute, useParams } from "@tanstack/solid-router";
import { FaSolidArrowRight, FaSolidRefresh } from "solid-icons/fa";
import { createSignal, Show, Suspense, useContext } from "solid-js";

import { appcontext } from "#/api";
import { accountBalanceQuery, refreshAccountBalances, useAccount, useAccountBalances } from "#/api/account";
import { AccountAssetSummary } from "#/components/account/asset/summary";
import { AccountTxPending } from "#/components/account/tx/pending";
import { AssetAmount } from "#/components/asset/amount";
import { Modal } from "#/components/dialog";
import { button } from "#/components/input/button";
import { FormattedTime } from "#/components/time";
import { narrow } from "#/utils/narrow";
import { ReceiveQR } from "#/views/receive/qr";

export const Route = createFileRoute("/acc/$account/")({
  component: () => {
    const params = useParams({ from: "/acc/$account" });
    const { displayCurrency: [displayCurrency] } = useContext(appcontext);
    // const balance = useAccountBalance(params().account);
    const account_identity = Number.parseInt(params().account);
    const balanceQuery = useAccountBalances(() => accountBalanceQuery(account_identity, displayCurrency()));
    const [refreshingBalances, setRefreshingBalances] = createSignal(false);
    const account = useAccount(() => ({ path: { account_identity } }));

    const refreshBalances = async () => {
      setRefreshingBalances(true);

      try {
        await refreshAccountBalances({
          path: { account_identity },
          query: { display_currency: displayCurrency() },
        });
      }
      finally {
        setRefreshingBalances(false);
      }
    };

    return (
      <div class="space-y-4">
        <div class="grid grid-cols-1 xl:grid-cols-5 w-full gap-4">
          <div class="space-y-4 w-full xl:col-span-3">
            <div class="bg-surface p-4 rounded-md space-y-4">
              <div class="flex justify-between items-center">
                <div class="text-sm font-bold text-muted">
                  Total balance
                </div>
                <div class="text-muted text-sm flex items-center gap-2">
                  <Suspense fallback={<span>Loading...</span>}>
                    <Show when={balanceQuery.data} fallback={<span>Loading...</span>}>
                      {data => (
                        <span class="flex items-center gap-1">
                          <FormattedTime value={data().updated_at} prefix="Updated " />
                        </span>
                      )}
                    </Show>
                  </Suspense>
                  <Show when={!balanceQuery.isLoading || balanceQuery.data}>
                    <button
                      class={button({ variant: "ghost", size: "small", square: true })}
                      onClick={() => { void refreshBalances(); }}
                    >
                      <FaSolidRefresh classList={{
                        "size-3.5": true,
                        "animate-spin": refreshingBalances(),
                      }}
                      />
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
            <AccountTxPending account_identity={account_identity} />
          </div>
        </div>
      </div>
    );
  },
});
