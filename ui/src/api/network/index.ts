import { createQuery } from "@tanstack/solid-query";

import { api } from "#/api";

export const useNetwork = (network_id: string) => createQuery(() => ({
    queryKey: ["network", network_id],
    queryFn: async () => {
        const response = await api("/net/{network_id}", "get", {
            path: {
                network_id: Number.parseInt(network_id),
            },
        });

        return response.data;
    },
}));

export const useNetworks = () => createQuery(() => ({
    queryKey: ["networks"],
    queryFn: async () => {
        const response = await api("/net", "get", {});

        return response.data;
    },
}));
