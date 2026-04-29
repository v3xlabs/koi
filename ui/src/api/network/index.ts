
import { queryClient } from "../client";
import { createApi, createApiMutation } from "../query";
import { components } from "../schema.gen";

export type Network = components["schemas"]["Network"];
export type NetworkEndpoint = components["schemas"]["NetworkEndpoint"];

export const useNetwork = createApi("/net/{network_id}", "get", options => ["network", options.path.network_id.toString()]);
export const useNetworks = createApi("/net", "get", () => ["networks"]);
export const useNetworkPresets = createApi("/net/presets", "get", () => ["network-presets"]);
export const useCreateNetwork = createApiMutation("/net", "post", {
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ["networks"] });
    },
});
export const useUpdateNetwork = createApiMutation("/net/{network_id}", "put", {
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ["networks"] });
    },
});
export const useDeleteNetwork = createApiMutation("/net/{network_id}", "delete", {
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ["networks"] });
    },
});

export const useNetworkEndpointNextId = createApi("/net/{network_id}/endpoints/next-id", "get", () => ["network-endpoint-next-id"]);

export const useNetworkEndpoints = createApi("/net/{network_id}/endpoints", "get", options => ["network-endpoints", options.path.network_id.toString()]);
export const useCreateNetworkEndpoint = createApiMutation("/net/{network_id}/endpoints", "post", {
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ["network-endpoints"] });
    },
});
export const useNetworkEndpoint = createApi("/net/{network_id}/endpoints/{endpoint_id}", "get", options => ["network-endpoint", options.path.network_id.toString(), options.path.endpoint_id.toString()]);
export const useUpdateNetworkEndpoint = createApiMutation("/net/{network_id}/endpoints/{endpoint_id}", "put", {
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ["network-endpoints"] });
    },
});
export const useDeleteNetworkEndpoint = createApiMutation("/net/{network_id}/endpoints/{endpoint_id}", "delete", {
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ["network-endpoints"] });
    },
});
export const useNetworkEndpointStatus = createApi("/net/{network_id}/endpoints/{endpoint_id}/status", "get", options => ["network-endpoint-status", options.path.network_id.toString(), options.path.endpoint_id.toString()]);
