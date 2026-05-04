import { queryClient } from "../client";
import { createApi } from "../query";

export const quoterKeys = {
    all: ["quoters"] as const,
    detail: (quoter_identity: string) => ["quoters", quoter_identity] as const,
};

export const useQuoters = createApi("/quoter", "get", () => quoterKeys.all, {
    onData: data => data.quoters.forEach(quoter => queryClient.setQueryData(quoterKeys.detail(quoter.quoter_identity), quoter)),
});

export const useQuoter = createApi("/quoter/{quoter_identity}", "get", options => quoterKeys.detail(options.path.quoter_identity));
