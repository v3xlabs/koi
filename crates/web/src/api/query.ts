/* eslint-disable @typescript-eslint/no-explicit-any */
import { createMutation, createQuery, MutationOptions, SolidQueryOptions, UseQueryResult } from "@tanstack/solid-query";
import { PathMethods } from "openapi-hooks";

import { api } from ".";
import { queryClient } from "./client";
import { paths } from "./schema.gen";

export type CreateApiOptions<T> = () => T;

export type ApiQueryKey = readonly unknown[];

type QT<T, Q, R> = T extends undefined ? R : Q extends undefined ? R : (R | undefined);

type ApiPropsTwo<TOptions, QueryKeyFn> = QT<TOptions, QueryKeyFn, () => TOptions>;

type ApiQueryOptions<TData> = Omit<Partial<SolidQueryOptions<TData, Error, TData, ApiQueryKey>>, "queryKey" | "queryFn" | "initialData">;
type ApiQueryConfig<TData> = SolidQueryOptions<TData, Error, TData, ApiQueryKey>;
type ApiConfig<TOptions, TData> = {
    onData?: (data: TData, options: TOptions) => void;
};

type ApiQueryResult<TOptions, TData, QueryKeyFn> = {
    (propstwo?: ApiPropsTwo<TOptions, QueryKeyFn>, extraOptions?: ApiQueryOptions<TData>): UseQueryResult<TData, Error>;
    options: (options: TOptions, extraOptions?: ApiQueryOptions<TData>) => ApiQueryConfig<TData>;
    ensure: (options: TOptions, extraOptions?: ApiQueryOptions<TData>) => Promise<TData>;
};

type ApiSuccessData<TPath extends keyof paths, TMethod extends PathMethods<paths, TPath>> = NonNullable<paths[TPath][TMethod]> extends { responses: { 200: { content: infer TContent; }; }; }
    ? TContent[keyof TContent]
    : never;

export const createApi = <
    TPath extends keyof paths,
    TMethod extends PathMethods<paths, TPath>,
    TOptions extends Parameters<typeof api<TPath, TMethod>>[2],
    TData extends ApiSuccessData<TPath, TMethod> = ApiSuccessData<TPath, TMethod>,
    QueryKeyFn extends ((options: TOptions) => ApiQueryKey) | undefined = ((options: TOptions) => ApiQueryKey),
>(path: TPath, method: TMethod, queryKeygen?: QueryKeyFn, config: ApiConfig<TOptions, TData> = {}): ApiQueryResult<TOptions, TData, QueryKeyFn> => {
    const options = (options: TOptions, extraOptions: ApiQueryOptions<TData> = {}) => {
        const queryKey = queryKeygen?.(options);

        if (!queryKey) {
            throw new Error("Query key is required");
        }

        return {
            queryKey,
            queryFn: async () => {
                const response = await api(path, method, options as any);

                if (response.status !== 200) {
                    if (
                        extraOptions.throwOnError)
                        throw new Error(response.status.toString());

                    return undefined;
                }

                const data = response.data as TData;

                config.onData?.(data, options);

                return data;
            },
            ...extraOptions,
        };
    };

    const query = (propstwo: ApiPropsTwo<TOptions, QueryKeyFn> = undefined as ApiPropsTwo<TOptions, QueryKeyFn>, extraOptions: ApiQueryOptions<TData> = {}) => createQuery(() => {
        const apiOptions: TOptions = propstwo ? propstwo() : {} as TOptions;

        return options(apiOptions, extraOptions);
    });

    query.options = options;
    query.ensure = (apiOptions: TOptions, extraOptions: ApiQueryOptions<TData> = {}) => queryClient.ensureQueryData(options(apiOptions, extraOptions) as any) as Promise<TData>;

    return query;
};

type ApiPropsThree<TOptions, TTOptions> = (props: TTOptions) => TOptions;

export const createApiMutation = <
    TPath extends keyof paths,
    TMethod extends PathMethods<paths, TPath>,
    TOptions extends Parameters<typeof api<TPath, TMethod>>[2],
    TData extends ApiSuccessData<TPath, TMethod> = ApiSuccessData<TPath, TMethod>,
    TTTOptions extends Omit<MutationOptions<TData, Error, any, any>, "mutationFn"> = Omit<MutationOptions<TData, Error, any, any>, "mutationFn">,
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
