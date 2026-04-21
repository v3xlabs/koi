/* eslint-disable @typescript-eslint/no-explicit-any */
import { createQuery } from "@tanstack/solid-query";
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
    QueryKeyFn extends ((options: TOptions) => string[]) | undefined = ((options: TOptions) => string[]),
>(path: TPath, method: TMethod, queryKeygen?: QueryKeyFn) => {
    console.log("setup");

    return (propstwo: ApiPropsTwo<TOptions, QueryKeyFn> = undefined as ApiPropsTwo<TOptions, QueryKeyFn>) => {
        console.log("use");

        return createQuery(() => {
            console.log("hi");
            const options: TOptions = propstwo ? propstwo() : {} as TOptions;

            console.log("hiz");
            const queryKey = queryKeygen?.(options);

            console.log({ queryKey });

            if (!queryKey) {
                console.log("query key is required");
                throw new Error("Query key is required");
            }

            console.log("return");

            return {
                queryKey,
                queryFn: async () => {
                    const response = await api(path, method, options as any);

                    return response.data;
                },
            };
        });
    };
};
