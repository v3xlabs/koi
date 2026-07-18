import type { Quoter, QuoterCreate, QuoterDiscovery, QuoterDiscoveryResponse, QuoterUpdate } from "../bindings.gen";
import { queryClient } from "../client";
import { createRpcMutation, createRpcQuery, requireOptions } from "../query";
import { rpc } from "../rpc.gen";

type Path = { path: { quoter_identity: string; }; };

export const quoterKeys = {
    all: ["quoters"] as const,
    detail: (quoterIdentity: string) => ["quoters", quoterIdentity] as const,
    discover: (tokenA: string, tokenB: string | undefined | null) => ["quoter-discover", tokenA, tokenB ?? "-"] as const,
};

export const useQuoters = createRpcQuery<void, { quoters: Quoter[]; }>(
    async () => ({ quoters: await rpc.quoterList() }),
    () => quoterKeys.all,
    {
        onData: data => data.quoters.forEach(quoter => queryClient.setQueryData(quoterKeys.detail(quoter.quoter_identity), quoter)),
    },
);
export const useQuoter = createRpcQuery<Path, Quoter>(
    options => rpc.quoterGet(requireOptions(options).path.quoter_identity),
    options => quoterKeys.detail(requireOptions(options).path.quoter_identity),
);
export const useCreateQuoter = createRpcMutation<{ data: QuoterCreate; }, Quoter>(
    options => rpc.quoterCreate(options.data),
    {
        onSuccess: (quoter) => {
            void queryClient.invalidateQueries({ queryKey: quoterKeys.all });
            void queryClient.invalidateQueries({ queryKey: quoterKeys.detail(quoter.quoter_identity) });
        },
    },
);
export const useUpdateQuoter = createRpcMutation<Path & { data: QuoterUpdate; }, Quoter>(
    options => rpc.quoterUpdate(options.path.quoter_identity, options.data),
    {
        onSuccess: (quoter) => {
            void queryClient.invalidateQueries({ queryKey: quoterKeys.all });
            void queryClient.invalidateQueries({ queryKey: quoterKeys.detail(quoter.quoter_identity) });
        },
    },
);
export const useDiscoverQuoter = createRpcQuery<{ data: QuoterDiscovery; }, QuoterDiscoveryResponse>(
    options => rpc.quoterDiscover(requireOptions(options).data),
    (options) => {
        const value = requireOptions(options).data;

        return quoterKeys.discover(value.token_a, value.token_b);
    },
);

export { type Quoter, type QuoterConfig, type QuoterCreate, type QuoterUpdate, type UniswapV2Pair, type UniswapV3Pool } from "../bindings.gen";
