import { FiCheck, FiX } from "solid-icons/fi";
import { createMemo, For, Show } from "solid-js";

import { useApproveWalletRequest, useRejectWalletRequest, useWalletRequests, WalletRequest } from "#/api/wallet-request";
import { button } from "#/components/input/button";

export const PendingWalletRequests = () => {
  const query = useWalletRequests();
  const approve = useApproveWalletRequest(props => ({
    path: { request_id: props.request_id },
  }));
  const reject = useRejectWalletRequest(props => ({
    path: { request_id: props.request_id },
    data: {},
    contentType: "application/json; charset=utf-8",
  }));
  const requests = createMemo(() => query.data?.requests ?? []);

  return (
    <Show when={requests().length > 0}>
      <div class="fixed inset-x-3 bottom-3 z-50 mx-auto w-[min(34rem,calc(100vw-1.5rem))] rounded-lg border border-border bg-surface shadow-xl">
        <div class="border-b border-border px-4 py-3">
          <div class="text-sm font-semibold">Pending wallet requests</div>
        </div>
        <div class="max-h-[min(28rem,70vh)] overflow-y-auto p-3">
          <div class="space-y-3">
            <For each={requests()}>
              {request => (
                <div class="rounded-md border border-border bg-background p-3">
                  <div class="flex min-w-0 items-start justify-between gap-3">
                    <div class="min-w-0">
                      <div class="truncate text-sm font-medium">{requestTitle(request)}</div>
                      <div class="mt-1 truncate text-xs text-muted">{request.method}</div>
                    </div>
                  </div>
                  <pre class="mt-3 max-h-36 overflow-auto rounded bg-surface-alt p-2 text-xs leading-relaxed text-muted">
                    {JSON.stringify(request.params, null, 2)}
                  </pre>
                  <div class="mt-3 flex justify-end gap-2">
                    <button
                      type="button"
                      class={button({ variant: "ghost", class: "text-sm" })}
                      disabled={reject.isPending || approve.isPending}
                      onClick={() => reject.mutate({ request_id: request.request_id })}
                    >
                      <FiX />
                      Reject
                    </button>
                    <button
                      type="button"
                      class={button({ variant: "primary", class: "text-sm" })}
                      disabled={reject.isPending || approve.isPending}
                      onClick={() => approve.mutate({ request_id: request.request_id })}
                    >
                      <FiCheck />
                      Approve
                    </button>
                  </div>
                </div>
              )}
            </For>
          </div>
        </div>
      </div>
    </Show>
  );
};

const requestTitle = (request: WalletRequest) => {
  switch (request.kind) {
    case "permission": {
      return "Account permission";
    }
    case "asset": {
      return "Asset request";
    }
    case "signature": {
      return "Signature request";
    }
    case "transaction": {
      return "Transaction request";
    }
    case "network": {
      return "Network request";
    }
    case "read": {
      return "Read request";
    }
    case "unknown": {
      return "Unknown wallet request";
    }
  }
};
