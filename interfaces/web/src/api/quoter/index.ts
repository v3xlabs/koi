import { queryClient } from "../client";
import { createApi, createApiMutation } from "../query";
import { components } from "../schema.gen";

export type Quoter = components["schemas"]["Quoter"];
export type QuoterConfig = components["schemas"]["QuoterConfig"];
export type QuoterCreate = components["schemas"]["QuoterCreate"];
export type QuoterUpdate = components["schemas"]["QuoterUpdate"];
export type UniswapV3Pool = components["schemas"]["UniswapV3Pool"];
export type UniswapV2Pair = components["schemas"]["UniswapV2Pair"];

export const quoterKeys = {
    all: ["quoters"] as const,
    detail: (quoter_identity: string) => ["quoters", quoter_identity] as const,
    discover: (token_a: string, token_b: string | undefined) => ["quoter-discover", token_a, token_b ?? "-"] as const,
};

export const useQuoters = createApi("/quoter", "get", () => quoterKeys.all, {
    onData: data => data.quoters.forEach(quoter => queryClient.setQueryData(quoterKeys.detail(quoter.quoter_identity), quoter)),
});

export const useQuoter = createApi("/quoter/{quoter_identity}", "get", options => quoterKeys.detail(options.path.quoter_identity));

export const useCreateQuoter = createApiMutation("/quoter", "post", {
    onSuccess: (quoter) => {
        queryClient.invalidateQueries({ queryKey: quoterKeys.all });
        queryClient.invalidateQueries({ queryKey: quoterKeys.detail(quoter.quoter_identity) });
    },
});

export const useUpdateQuoter = createApiMutation("/quoter/{quoter_identity}", "put", {
    onSuccess: (quoter) => {
        queryClient.invalidateQueries({ queryKey: quoterKeys.all });
        queryClient.invalidateQueries({ queryKey: quoterKeys.detail(quoter.quoter_identity) });
    },
});

export const useDiscoverQuoter = createApi("/quoter/discover", "post", options => quoterKeys.discover(options.data?.token_a, options.data?.token_b));
