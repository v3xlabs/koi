import type { Asset, AssetMetadataDiscovery, AssetUpdate } from "../bindings.gen";
import { queryClient } from "../client";
import { createRpcMutation, createRpcQuery, requireOptions } from "../query";
import { rpc } from "../rpc.gen";

type Path = { path: { asset_identity: string; }; };

export const assetKeys = {
    all: ["assets"] as const,
    detail: (assetIdentity: string) => ["assets", assetIdentity] as const,
    discovery: (assetIdentity: string) => ["asset-discovery", assetIdentity] as const,
    quote: (assetIdentity: string) => ["quote", assetIdentity] as const,
};

export const useAssets = createRpcQuery<void, { assets: Asset[]; }>(
    async () => ({ assets: await rpc.assetList() }),
    () => assetKeys.all,
    {
        onData: data => data.assets.forEach(asset => queryClient.setQueryData(assetKeys.detail(asset.asset_identity), asset)),
    },
);
export const useAsset = createRpcQuery<Path, Asset>(
    options => rpc.assetGet({ asset_identity: requireOptions(options).path.asset_identity }),
    options => assetKeys.detail(requireOptions(options).path.asset_identity),
);
export const useCreateAsset = createRpcMutation<{ data: Asset; }, Asset>(
    options => rpc.assetCreate({ input: options.data }),
    {
        onSuccess: (asset) => {
            void queryClient.invalidateQueries({ queryKey: assetKeys.all });
            void queryClient.invalidateQueries({ queryKey: assetKeys.detail(asset.asset_identity) });
        },
    },
);
export const useUpdateAsset = createRpcMutation<Path & { data: AssetUpdate; }, Asset>(
    options => rpc.assetUpdate({ asset_identity: options.path.asset_identity, input: options.data }),
    {
        onSuccess: (asset) => {
            void queryClient.invalidateQueries({ queryKey: assetKeys.all });
            void queryClient.invalidateQueries({ queryKey: assetKeys.detail(asset.asset_identity) });
        },
    },
);
export const useDeleteAsset = createRpcMutation<Path, null>(
    options => rpc.assetDelete({ asset_identity: options.path.asset_identity }),
    {
        onSuccess: (_, options) => {
            void queryClient.invalidateQueries({ queryKey: assetKeys.all });
            queryClient.removeQueries({ queryKey: assetKeys.detail(options.path.asset_identity) });
        },
    },
);
export const useAssetMetadataDiscovery = createRpcQuery<Path, AssetMetadataDiscovery>(
    options => rpc.assetDiscoverMetadata({ asset_identity: requireOptions(options).path.asset_identity }),
    options => assetKeys.discovery(requireOptions(options).path.asset_identity),
);
export const useAssetQuote = createRpcQuery<Path & { query?: { display_asset?: string; }; }, string>(
    (options) => {
        const value = requireOptions(options);

        return rpc.assetQuote({ asset_identity: value.path.asset_identity, display_asset: value.query?.display_asset });
    },
    options => assetKeys.quote(requireOptions(options).path.asset_identity),
);

export { type Asset, type AssetUpdate } from "../bindings.gen";
