import { createQuery } from "@tanstack/solid-query";

import { api } from "#/api";

export const useAccount = (account_id: string) => createQuery(() => ({
    queryKey: ["account", account_id],
    queryFn: async () => {
        const response = await api("/acc/{account_id}", "get", {
            path: {
                account_id,
            },
        });

        return response.data;
    },
}));

export const useAccounts = () => createQuery(() => ({
    queryKey: ["accounts"],
    queryFn: async () => {
        const response = await api("/acc", "get", {});

        return response.data;
    },
}));
