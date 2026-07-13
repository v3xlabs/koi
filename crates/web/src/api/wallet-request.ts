import { queryClient } from "./client";
import { createApi, createApiMutation } from "./query";
import { components } from "./schema.gen";

export type WalletRequest = components["schemas"]["FrontendWalletRequest"];

export const walletRequestKeys = {
  all: ["wallet-requests"] as const,
};

const invalidateWalletRequests = () => {
  queryClient.invalidateQueries({ queryKey: walletRequestKeys.all });
};

export const useWalletRequests = createApi("/wallet-requests", "get", () => walletRequestKeys.all);

export const useApproveWalletRequest = createApiMutation(
  "/wallet-requests/{request_id}/approve",
  "post",
  {
    onSuccess: invalidateWalletRequests,
  },
);

export const useRejectWalletRequest = createApiMutation(
  "/wallet-requests/{request_id}/reject",
  "post",
  {
    onSuccess: invalidateWalletRequests,
  },
);
