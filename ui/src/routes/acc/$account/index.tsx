import { createFileRoute, useParams } from "@tanstack/solid-router";
import { FaSolidArrowRight, FaSolidRefresh } from "solid-icons/fa";
import { Show, Suspense } from "solid-js";

import { useAccount } from "#/api/account";
import { useAccountBalance } from "#/api/account/balance";
import { AssetAmount } from "#/components/asset/amount";
import { Modal } from "#/components/dialog";
import { narrow } from "#/utils/narrow";
import { ReceiveQR } from "#/views/receive/qr";

export const Route = createFileRoute("/acc/$account/")({
  component: () => {
    const params = useParams({ from: "/acc/$account" });
    const balance = useAccountBalance(params().account);
    const account = useAccount(params().account);

    return (
      <div class="p-4 grid grid-cols-5 w-full gap-4">
        <div class="bg-surface p-4 col-span-3 rounded-md space-y-4">
          <div class="flex justify-between items-center">
            <div class="text-sm font-bold text-muted">
              Total balance
            </div>
            <div class="text-muted text-sm flex items-center gap-2">
              <Suspense>
                <span>
                  Updated
                  {" "}
                  {balance.data?.updated_at.toLocaleTimeString()}
                </span>
              </Suspense>
              <Show when={!balance.isLoading}>
                <button
                  classList={{
                    "cursor-pointer": !balance.isRefetching,
                    "animate-spin": balance.isRefetching,
                  }}
                  onClick={() => balance.refetch()}
                >
                  <FaSolidRefresh class="w-3.5 h-3.5 text-primary-foreground" />
                </button>
              </Show>
            </div>
          </div>
          <div class="flex justify-between items-center">
            <div class="text-4xl font-bold tabular-nums">
              <Suspense fallback={<span class="text-muted">Loading...</span>}>
                <Show when={balance.data}>
                  {data => (
                    <AssetAmount amount={() => data().balance} asset={() => data().asset} />
                  )}
                </Show>
              </Suspense>
            </div>
            <div class="flex gap-2">
              <button
                class="bg-primary hover:bg-primary-hover text-primary-foreground w-full rounded-md py-2 px-4 flex items-center gap-2 cursor-pointer justify-center text-sm font-bold"
              >
                <FaSolidArrowRight class="-rotate-45" />
                Send
              </button>
              <Show when={narrow(() => account.data?.metadata, x => "evm_address" in x)}>
                {x => (
                  <ReceiveQR address={() => x().evm_address}>
                    <Modal.Trigger
                      class="bg-secondary hover:bg-secondary-hover text-primary-foreground w-full rounded-md py-2 px-4 flex items-center gap-2 cursor-pointer justify-center text-sm font-bold"
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
        <div class="bg-surface p-4 col-span-2 rounded-md">
          <div>
            <div>
              Pending transactions
            </div>
          </div>
        </div>
      </div>
    );
  },
});
