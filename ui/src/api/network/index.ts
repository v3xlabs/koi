
import { queryClient } from "../client";
import { createApi, createApiMutation } from "../query";
import { components } from "../schema.gen";

export type Network = components["schemas"]["Network"];
export type NetworkEndpoint = components["schemas"]["NetworkEndpoint"];

export const networkKeys = {
    all: ["networks"] as const,
    detail: (network_identity: number | string) => ["network", network_identity.toString()] as const,
    presets: ["network-presets"] as const,
    endpoints: (network_identity: number | string) => ["network-endpoints", network_identity.toString()] as const,
    endpoint: (network_identity: number | string, endpoint_identity: number | string) => ["network-endpoint", network_identity.toString(), endpoint_identity.toString()] as const,
    endpointNextId: (network_identity: number | string) => ["network-endpoint-next-id", network_identity.toString()] as const,
    endpointStatus: (network_identity: number | string, endpoint_identity: number | string) => ["network-endpoint-status", network_identity.toString(), endpoint_identity.toString()] as const,
};

export const useNetwork = createApi("/net/{network_identity}", "get", options => networkKeys.detail(options.path.network_identity));
export const useNetworks = createApi("/net", "get", () => networkKeys.all);
export const useNetworkPresets = createApi("/net/presets", "get", () => networkKeys.presets);
export const useCreateNetwork = createApiMutation("/net", "post", {
    onSuccess: (network) => {
        queryClient.invalidateQueries({ queryKey: networkKeys.all });
        queryClient.invalidateQueries({ queryKey: networkKeys.presets });
        queryClient.invalidateQueries({ queryKey: networkKeys.detail(network.network_identity) });
    },
});
export const useUpdateNetwork = createApiMutation("/net/{network_identity}", "put", {
    onSuccess: (network) => {
        queryClient.invalidateQueries({ queryKey: networkKeys.all });
        queryClient.invalidateQueries({ queryKey: networkKeys.detail(network.network_identity) });
    },
});
export const useDeleteNetwork = createApiMutation("/net/{network_identity}", "delete", {
    onSuccess: (_, variables) => {
        queryClient.invalidateQueries({ queryKey: networkKeys.all });
        queryClient.invalidateQueries({ queryKey: networkKeys.presets });
        queryClient.removeQueries({ queryKey: networkKeys.detail(variables.network_identity) });
        queryClient.removeQueries({ queryKey: networkKeys.endpoints(variables.network_identity) });
    },
});

export const useNetworkEndpointNextId = createApi("/net/{network_identity}/endpoints/next-id", "get", options => networkKeys.endpointNextId(options.path.network_identity));

export const useNetworkEndpoints = createApi("/net/{network_identity}/endpoints", "get", options => networkKeys.endpoints(options.path.network_identity));
export const useCreateNetworkEndpoint = createApiMutation("/net/{network_identity}/endpoints", "post", {
    onSuccess: (endpoint) => {
        queryClient.invalidateQueries({ queryKey: networkKeys.endpoints(endpoint.network_identity) });
        queryClient.invalidateQueries({ queryKey: networkKeys.endpointNextId(endpoint.network_identity) });
        queryClient.invalidateQueries({ queryKey: networkKeys.endpoint(endpoint.network_identity, endpoint.endpoint_identity) });
    },
});
export const useNetworkEndpoint = createApi("/net/{network_identity}/endpoints/{endpoint_identity}", "get", options => networkKeys.endpoint(options.path.network_identity, options.path.endpoint_identity));
export const useUpdateNetworkEndpoint = createApiMutation("/net/{network_identity}/endpoints/{endpoint_identity}", "put", {
    onSuccess: (endpoint) => {
        queryClient.invalidateQueries({ queryKey: networkKeys.endpoints(endpoint.network_identity) });
        queryClient.invalidateQueries({ queryKey: networkKeys.endpoint(endpoint.network_identity, endpoint.endpoint_identity) });
        queryClient.invalidateQueries({ queryKey: networkKeys.endpointStatus(endpoint.network_identity, endpoint.endpoint_identity) });
    },
});
export const useDeleteNetworkEndpoint = createApiMutation("/net/{network_identity}/endpoints/{endpoint_identity}", "delete", {
    onSuccess: (_, variables) => {
        queryClient.invalidateQueries({ queryKey: networkKeys.endpoints(variables.network_identity) });
        queryClient.invalidateQueries({ queryKey: networkKeys.endpointNextId(variables.network_identity) });
        queryClient.removeQueries({ queryKey: networkKeys.endpoint(variables.network_identity, variables.endpoint_identity) });
        queryClient.removeQueries({ queryKey: networkKeys.endpointStatus(variables.network_identity, variables.endpoint_identity) });
    },
});
export const useNetworkEndpointStatus = createApi("/net/{network_identity}/endpoints/{endpoint_identity}/status", "get", options => networkKeys.endpointStatus(options.path.network_identity, options.path.endpoint_identity));
