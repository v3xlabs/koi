import type { Network, NetworkEndpoint, NetworkEndpointCreate, NetworkEndpointUpdate, NetworkMetadataDiscovery, NetworkUpdate, RpcPoolStats, RpcStatus } from "../bindings.gen";
import { queryClient } from "../client";
import { createRpcMutation, createRpcQuery, requireOptions } from "../query";
import { rpc } from "../rpc.gen";

type NetworkPath = { path: { network_identity: number; }; };
type EndpointPath = { path: { network_identity: number; endpoint_identity: number; }; };

export const networkKeys = {
    all: ["networks"] as const,
    detail: (networkIdentity: number | string) => ["network", networkIdentity.toString()] as const,
    discovery: (networkIdentity: number | string) => ["network-discovery", networkIdentity.toString()] as const,
    presets: ["network-presets"] as const,
    endpoints: (networkIdentity: number | string) => ["network-endpoints", networkIdentity.toString()] as const,
    endpoint: (networkIdentity: number | string, endpointIdentity: number | string) => ["network-endpoint", networkIdentity.toString(), endpointIdentity.toString()] as const,
    endpointStatus: (networkIdentity: number | string, endpointIdentity: number | string) => ["network-endpoint-status", networkIdentity.toString(), endpointIdentity.toString()] as const,
    rpc: (networkIdentity: number | string) => ["network-rpc", networkIdentity.toString()] as const,
};

const invalidateNetwork = (network: Network) => {
    void queryClient.invalidateQueries({ queryKey: networkKeys.all });
    void queryClient.invalidateQueries({ queryKey: networkKeys.detail(network.network_identity) });
};

const invalidateEndpoint = (endpoint: NetworkEndpoint) => {
    void queryClient.invalidateQueries({ queryKey: networkKeys.endpoints(endpoint.network_identity) });
    void queryClient.invalidateQueries({ queryKey: networkKeys.endpoint(endpoint.network_identity, endpoint.endpoint_identity) });
};

export const useNetwork = createRpcQuery<NetworkPath, Network>(
    options => rpc.networkGet({ network_identity: requireOptions(options).path.network_identity }),
    options => networkKeys.detail(requireOptions(options).path.network_identity),
);
export const useNetworkMetadataDiscovery = createRpcQuery<NetworkPath, NetworkMetadataDiscovery>(
    options => rpc.networkDiscover({ network_identity: requireOptions(options).path.network_identity }),
    options => networkKeys.discovery(requireOptions(options).path.network_identity),
);
export const useNetworkRpcStats = createRpcQuery<NetworkPath, RpcPoolStats>(
    options => rpc.networkStats({ network_identity: requireOptions(options).path.network_identity }),
    options => networkKeys.rpc(requireOptions(options).path.network_identity),
);
export const useNetworks = createRpcQuery<void, { networks: Network[]; }>(
    async () => ({ networks: await rpc.networkList() }),
    () => networkKeys.all,
    {
        onData: data => data.networks.forEach(network => queryClient.setQueryData(networkKeys.detail(network.network_identity), network)),
    },
);
export const useNetworkPresets = createRpcQuery<void, Network[]>(
    () => rpc.networkPresets(),
    () => networkKeys.presets,
);
export const useCreateNetwork = createRpcMutation<{ data: Network; }, Network>(
    options => rpc.networkCreate({ input: options.data }),
    { onSuccess: invalidateNetwork },
);
export const useUpdateNetwork = createRpcMutation<NetworkPath & { data: NetworkUpdate; }, Network>(
    options => rpc.networkUpdate({ network_identity: options.path.network_identity, input: options.data }),
    { onSuccess: invalidateNetwork },
);
export const useDeleteNetwork = createRpcMutation<NetworkPath, null>(
    options => rpc.networkDelete({ network_identity: options.path.network_identity }),
    {
        onSuccess: (_, options) => {
            void queryClient.invalidateQueries({ queryKey: networkKeys.all });
            queryClient.removeQueries({ queryKey: networkKeys.detail(options.path.network_identity) });
            queryClient.removeQueries({ queryKey: networkKeys.endpoints(options.path.network_identity) });
        },
    },
);
export const useNetworkEndpoints = createRpcQuery<NetworkPath, NetworkEndpoint[]>(
    options => rpc.endpointList({ network_identity: requireOptions(options).path.network_identity }),
    options => networkKeys.endpoints(requireOptions(options).path.network_identity),
    {
        onData: endpoints => endpoints.forEach(endpoint => queryClient.setQueryData(networkKeys.endpoint(endpoint.network_identity, endpoint.endpoint_identity), endpoint)),
    },
);
export const useCreateNetworkEndpoint = createRpcMutation<NetworkPath & { data: NetworkEndpointCreate; }, NetworkEndpoint>(
    options => rpc.endpointCreate({ network_identity: options.path.network_identity, input: options.data }),
    {
        onSuccess: (endpoint) => {
            invalidateEndpoint(endpoint);
        },
    },
);
export const useNetworkEndpoint = createRpcQuery<EndpointPath, NetworkEndpoint>(
    (options) => {
        const value = requireOptions(options);

        return rpc.endpointGet({ network_identity: value.path.network_identity, endpoint_identity: value.path.endpoint_identity });
    },
    (options) => {
        const value = requireOptions(options);

        return networkKeys.endpoint(value.path.network_identity, value.path.endpoint_identity);
    },
);
export const useUpdateNetworkEndpoint = createRpcMutation<EndpointPath & { data: NetworkEndpointUpdate; }, NetworkEndpoint>(
    options => rpc.endpointUpdate({ network_identity: options.path.network_identity, endpoint_identity: options.path.endpoint_identity, input: options.data }),
    {
        onSuccess: (endpoint) => {
            invalidateEndpoint(endpoint);
            void queryClient.invalidateQueries({ queryKey: networkKeys.endpointStatus(endpoint.network_identity, endpoint.endpoint_identity) });
            void queryClient.invalidateQueries({ queryKey: networkKeys.rpc(endpoint.network_identity) });
        },
    },
);
export const useDeleteNetworkEndpoint = createRpcMutation<EndpointPath, null>(
    options => rpc.endpointDelete({ network_identity: options.path.network_identity, endpoint_identity: options.path.endpoint_identity }),
    {
        onSuccess: (_, options) => {
            const { endpoint_identity, network_identity } = options.path;

            void queryClient.invalidateQueries({ queryKey: networkKeys.endpoints(network_identity) });
            queryClient.removeQueries({ queryKey: networkKeys.endpoint(network_identity, endpoint_identity) });
            queryClient.removeQueries({ queryKey: networkKeys.endpointStatus(network_identity, endpoint_identity) });
            void queryClient.invalidateQueries({ queryKey: networkKeys.rpc(network_identity) });
        },
    },
);
export const useNetworkEndpointStatus = createRpcQuery<EndpointPath, RpcStatus>(
    (options) => {
        const value = requireOptions(options);

        return rpc.endpointStatus({ network_identity: value.path.network_identity, endpoint_identity: value.path.endpoint_identity });
    },
    (options) => {
        const value = requireOptions(options);

        return networkKeys.endpointStatus(value.path.network_identity, value.path.endpoint_identity);
    },
);

export { type Network, type NetworkEndpoint, type NetworkEndpointCreate, type NetworkEndpointUpdate, type NetworkMetadataDiscovery, type RpcEndpointStats, type RpcPoolStats } from "../bindings.gen";
