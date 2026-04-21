import { createMutation } from "@tanstack/solid-query";

import { api } from "#/api";

import { createApi } from "../hook";
import { components } from "../schema.gen";

export type Network = components["schemas"]["Network"];

export const useNetwork = createApi("/net/{network_id}", "get", options => ["network", options.path.network_id.toString()]);
export const useNetworks = createApi("/net", "get", options => ["networks"]);

export const useCreateNetwork = () => createMutation(() => ({
    mutationFn: async (network: Network) => {
        const response = await api("/net", "post", {
            contentType: "application/json; charset=utf-8",
            data: network,
        });

        return response.data;
    },
}));
