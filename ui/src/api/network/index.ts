
import { queryClient } from "../client";
import { createApi, createApiMutation } from "../query";
import { components } from "../schema.gen";

export type Network = components["schemas"]["Network"];
export type NetworkEndpoint = components["schemas"]["NetworkEndpoint"];

export const useNetwork = createApi("/net/{network_identity}", "get", options => ["network", options.path.network_identity.toString()]);
export const useNetworks = createApi("/net", "get", () => ["networks"]);
export const useNetworkPresets = createApi("/net/presets", "get", () => ["network-presets"]);
export const useCreateNetwork = createApiMutation("/net", "post", {
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ["networks"] });
    },
});
export const useUpdateNetwork = createApiMutation("/net/{network_identity}", "put", {
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ["networks"] });
    },
});
export const useDeleteNetwork = createApiMutation("/net/{network_identity}", "delete", {
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ["networks"] });
    },
});

export const useNetworkEndpointNextId = createApi("/net/{network_identity}/endpoints/next-id", "get", () => ["network-endpoint-next-id"]);

export const useNetworkEndpoints = createApi("/net/{network_identity}/endpoints", "get", options => ["network-endpoints", options.path.network_identity.toString()]);
export const useCreateNetworkEndpoint = createApiMutation("/net/{network_identity}/endpoints", "post", {
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ["network-endpoints"] });
    },
});
export const useNetworkEndpoint = createApi("/net/{network_identity}/endpoints/{endpoint_identity}", "get", options => ["network-endpoint", options.path.network_identity.toString(), options.path.endpoint_identity.toString()]);
export const useUpdateNetworkEndpoint = createApiMutation("/net/{network_identity}/endpoints/{endpoint_identity}", "put", {
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ["network-endpoints"] });
    },
});
export const useDeleteNetworkEndpoint = createApiMutation("/net/{network_identity}/endpoints/{endpoint_identity}", "delete", {
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ["network-endpoints"] });
    },
});
export const useNetworkEndpointStatus = createApi("/net/{network_identity}/endpoints/{endpoint_identity}/status", "get", options => ["network-endpoint-status", options.path.network_identity.toString(), options.path.endpoint_identity.toString()]);
