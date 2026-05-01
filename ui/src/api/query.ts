/* eslint-disable @typescript-eslint/no-explicit-any */
import { createMutation, createQuery, MutationOptions, QueryOptions } from "@tanstack/solid-query";
import { PathMethods } from "openapi-hooks";

import { api } from ".";
import { paths } from "./schema.gen";

export type CreateApiOptions<T> = () => T;

export type ApiQueryKey = readonly unknown[];

type QT<T, Q, R> = T extends undefined ? R : Q extends undefined ? R : (R | undefined);

type ApiPropsTwo<TOptions, QueryKeyFn> = QT<TOptions, QueryKeyFn, () => TOptions>;

type ApiQueryOptions<TData> = Omit<Partial<QueryOptions<TData, Error, any, any>>, "queryKey" | "queryFn">;

export const createApi = <
    TPath extends keyof paths,
    TMethod extends PathMethods<paths, TPath>,
    TOptions extends Parameters<typeof api<TPath, TMethod>>[2],
    TData extends Awaited<ReturnType<typeof api<TPath, TMethod>>>["data"],
    QueryKeyFn extends ((options: TOptions) => ApiQueryKey) | undefined = ((options: TOptions) => ApiQueryKey),
>(path: TPath, method: TMethod, queryKeygen?: QueryKeyFn) =>
    (propstwo: ApiPropsTwo<TOptions, QueryKeyFn> = undefined as ApiPropsTwo<TOptions, QueryKeyFn>, extraOptions: ApiQueryOptions<TData> = {}) => createQuery(() => {
        const options: TOptions = propstwo ? propstwo() : {} as TOptions;

        const queryKey = queryKeygen?.(options);

        if (!queryKey) {
            throw new Error("Query key is required");
        }

        return {
            queryKey,
            queryFn: async () => {
                const response = await api(path, method, options as any);

                return response.data as TData;
            },
            ...extraOptions,
        };
    });

type ApiPropsThree<TOptions, TTOptions> = (props: TTOptions) => TOptions;

export const createApiMutation = <
    TPath extends keyof paths,
    TMethod extends PathMethods<paths, TPath>,
    TOptions extends Parameters<typeof api<TPath, TMethod>>[2],
    TData extends Awaited<ReturnType<typeof api<TPath, TMethod>>>["data"],
    TTTOptions extends Omit<MutationOptions<TData, Error, any, any>, "mutationFn">,
>(path: TPath, method: TMethod, extraOptions: Partial<TTTOptions> = {}) =>
    <TTOptions extends object>(propstwo: ApiPropsThree<TOptions, TTOptions>, extraExtraOptions: Partial<MutationOptions<TData, Error, TTOptions, any>> = {}) =>
        createMutation(() => ({
            mutationFn: async (props: TTOptions) => {
                const options: TOptions = propstwo(props);

                const response = await api(path, method, options as any);

                return response.data as TData;
            },
            ...extraOptions,
            ...extraExtraOptions,
        }));
