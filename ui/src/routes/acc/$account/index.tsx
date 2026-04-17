import { createFileRoute } from "@tanstack/solid-router";
import { FaSolidArrowRight, FaSolidRefresh } from "solid-icons/fa";

import { Modal } from "#/components/dialog";
import { ReceiveQR } from "#/views/receive/qr";

export const Route = createFileRoute("/acc/$account/")({
  component: () => (
    <div class="p-4 grid grid-cols-5 w-full gap-4">
      <div class="bg-surface p-4 col-span-3 rounded-md space-y-4">
        <div class="flex justify-between items-center">
          <div class="text-sm font-bold text-muted">
            Total balance
          </div>
          <div class="text-muted text-sm flex items-center gap-2">
            <span>
              Updated just now
            </span>
            <FaSolidRefresh class="w-3.5 h-3.5 text-primary-foreground" />
          </div>
        </div>
        <div class="flex justify-between items-center">
          <div class="text-4xl font-bold tabular-nums">
            <span class="text-foreground">
              $100,100
            </span>
            <span class="text-muted">.00</span>
          </div>
          <div class="flex gap-2">
            <button
              class="bg-primary hover:bg-primary-hover text-primary-foreground w-full rounded-md py-2 px-4 flex items-center gap-2 cursor-pointer justify-center text-sm font-bold"
            >
              <FaSolidArrowRight class="-rotate-45" />
              Send
            </button>
            <ReceiveQR address={() => "0x1234567890123456789012345678901234567890"}>
              <Modal.Trigger
                class="bg-secondary hover:bg-secondary-hover text-primary-foreground w-full rounded-md py-2 px-4 flex items-center gap-2 cursor-pointer justify-center text-sm font-bold"
              >
                <FaSolidArrowRight class="-rotate-225" />
                Receive
              </Modal.Trigger>
            </ReceiveQR>
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
  ),
});
