import { createMutation, createQuery, type MutationFunctionContext, type MutationOptions, type UndefinedInitialDataOptions, type UseQueryResult } from "@tanstack/solid-query";
import type { Accessor } from "solid-js";

import { queryClient } from "./client";

export type RpcQueryKey = readonly unknown[];
type RpcQueryConfig<TData> = ReturnType<UndefinedInitialDataOptions<TData, Error, TData, RpcQueryKey>>;
type RpcQueryOptions<TData> = Omit<Partial<RpcQueryConfig<TData>>, "queryKey" | "queryFn" | "initialData">;
type RpcQueryOptionsSource<TData> = RpcQueryOptions<TData> | Accessor<RpcQueryOptions<TData>>;
type RpcQueryBehavior<TOptions, TData> = { onData?: (data: TData, options: TOptions | undefined) => void; };

type RpcQueryResult<TOptions, TData> = {
    (props?: Accessor<TOptions>, extraOptions?: RpcQueryOptionsSource<TData>): UseQueryResult<TData, Error>;
    options: (options: TOptions, extraOptions?: RpcQueryOptions<TData>) => RpcQueryConfig<TData>;
    ensure: (options: TOptions, extraOptions?: RpcQueryOptions<TData>) => Promise<TData>;
};

export const requireOptions = <T>(options: T | undefined): T => {
    if (options === undefined) {
        throw new Error("RPC query options are required");
    }

    return options;
};

const resolveQueryOptions = <TData>(options: RpcQueryOptionsSource<TData>): RpcQueryOptions<TData> => (typeof options === "function" ? options() : options);

export const createRpcQuery = <TOptions, TData>(
    call: (options: TOptions | undefined) => Promise<TData>,
    queryKey: (options: TOptions | undefined) => RpcQueryKey,
    behavior: RpcQueryBehavior<TOptions, TData> = {},
): RpcQueryResult<TOptions, TData> => {
    const options = (rpcOptions: TOptions | undefined, extraOptions: RpcQueryOptions<TData> = {}): RpcQueryConfig<TData> => ({
        queryKey: queryKey(rpcOptions),
        queryFn: async () => {
            const data = await call(rpcOptions);

            behavior.onData?.(data, rpcOptions);

            return data;
        },
        ...extraOptions,
    });

    const query = (props?: Accessor<TOptions>, extraOptions: RpcQueryOptionsSource<TData> = {}) => createQuery(() => options(props?.(), resolveQueryOptions(extraOptions)));

    query.options = (rpcOptions: TOptions, extraOptions: RpcQueryOptions<TData> = {}) => options(rpcOptions, extraOptions);
    query.ensure = async (rpcOptions: TOptions, extraOptions: RpcQueryOptions<TData> = {}) => await queryClient.ensureQueryData<TData, Error, TData, RpcQueryKey>(options(rpcOptions, extraOptions));

    return query;
};

export const createRpcMutation = <TMapped extends object, TData>(
    call: (options: TMapped) => Promise<TData>,
    behavior: { onSuccess?: (data: TData, options: TMapped) => void; } = {},
) => <TVariables extends object = TMapped>(
    map: (variables: TVariables) => TMapped,
    overrides: Partial<MutationOptions<TData, Error, TVariables, unknown>> = {},
) => createMutation(() => ({
    mutationFn: async (variables: TVariables) => await call(map(variables)),
    ...overrides,
    onSuccess: async (data: TData, variables: TVariables, result: unknown, context: MutationFunctionContext) => {
        behavior.onSuccess?.(data, map(variables));
        await overrides.onSuccess?.(data, variables, result, context);
    },
}));
