
import { queryClient } from "../client";
import { createApi, createApiMutation } from "../query";
import { components } from "../schema.gen";

export type Network = components["schemas"]["Network"];

export const useNetwork = createApi("/net/{network_id}", "get", options => ["network", options.path.network_id.toString()]);
export const useNetworks = createApi("/net", "get", () => ["networks"]);
export const useNetworkPresets = createApi("/net/presets", "get", () => ["network-presets"]);
export const useCreateNetwork = createApiMutation("/net", "post", {
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ["networks"] });
    },
});
