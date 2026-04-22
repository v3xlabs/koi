/* eslint-disable @typescript-eslint/no-explicit-any */
import { createMutation, createQuery, MutationOptions, QueryOptions } from "@tanstack/solid-query";
import { PathMethods } from "openapi-hooks";

import { api } from ".";
import { paths } from "./schema.gen";

export type CreateApiOptions<T> = () => T;

type QT<T, Q, R> = T extends undefined ? R : Q extends undefined ? R : (R | undefined);

type ApiPropsTwo<TOptions, QueryKeyFn> = QT<TOptions, QueryKeyFn, () => TOptions>;

export const createApi = <
    TPath extends keyof paths,
    TMethod extends PathMethods<paths, TPath>,
    TOptions extends Parameters<typeof api<TPath, TMethod>>[2],
    TData extends Awaited<ReturnType<typeof api<TPath, TMethod>>>["data"],
    QueryKeyFn extends ((options: TOptions) => string[]) | undefined = ((options: TOptions) => string[]),
>(path: TPath, method: TMethod, queryKeygen?: QueryKeyFn) => {
    console.log("setup");

    return (propstwo: ApiPropsTwo<TOptions, QueryKeyFn> = undefined as ApiPropsTwo<TOptions, QueryKeyFn>, extraOptions: Partial<QueryOptions<TData, Error, any, any>> = {}) => createQuery(() => {
        const options: TOptions = propstwo ? propstwo() : {} as TOptions;

        const queryKey = queryKeygen?.(options);

        if (!queryKey) {
            console.log("query key is required");
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
};

type ApiPropsThree<TOptions, TTOptions> = (props: TTOptions) => TOptions;

export const createApiMutation = <
    TPath extends keyof paths,
    TMethod extends PathMethods<paths, TPath>,
    TOptions extends Parameters<typeof api<TPath, TMethod>>[2],
    TData extends Awaited<ReturnType<typeof api<TPath, TMethod>>>["data"],
    TTTOptions extends Omit<MutationOptions<TData, Error, any, any>, "mutationFn">,
>(path: TPath, method: TMethod, extraOptions: Partial<TTTOptions> = {}) =>
    <TTOptions extends object>(propstwo: ApiPropsThree<TOptions, TTOptions>) =>
        createMutation(() => ({
            mutationFn: async (props: TTOptions) => {
                const options: TOptions = propstwo(props);

                const response = await api(path, method, options as any);

                return response.data as TData;
            },
            ...extraOptions,
        }));
