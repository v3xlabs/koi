import { queryClient } from "../client";
import { createApi, createApiMutation } from "../query";
import { components } from "../schema.gen";

export type Asset = components["schemas"]["Asset"];
export type AssetUpdate = components["schemas"]["AssetUpdate"];

export const assetKeys = {
    all: ["assets"] as const,
    detail: (asset_identity: string) => ["assets", asset_identity] as const,
    discovery: (asset_identity: string) => ["asset-discovery", asset_identity] as const,
    quote: (asset_identity: string) => ["quote", asset_identity] as const,
};

export const useAssets = createApi("/asset", "get", () => assetKeys.all, {
    onData: data => data.assets.forEach(asset => queryClient.setQueryData(assetKeys.detail(asset.asset_identity), asset)),
});
export const useAsset = createApi("/asset/{asset_identity}", "get", options => assetKeys.detail(options.path.asset_identity));
export const useCreateAsset = createApiMutation("/asset", "post", {
    onSuccess: (asset) => {
        queryClient.invalidateQueries({ queryKey: assetKeys.all });
        queryClient.invalidateQueries({ queryKey: assetKeys.detail(asset.asset_identity) });
    },
});
export const useUpdateAsset = createApiMutation("/asset/{asset_identity}", "put", {
    onSuccess: (asset) => {
        queryClient.invalidateQueries({ queryKey: assetKeys.all });
        queryClient.invalidateQueries({ queryKey: assetKeys.detail(asset.asset_identity) });
    },
});
export const useDeleteAsset = createApiMutation("/asset/{asset_identity}", "delete", {
    onSuccess: (_, variables) => {
        queryClient.invalidateQueries({ queryKey: assetKeys.all });
        queryClient.removeQueries({ queryKey: assetKeys.detail(variables.asset_identity) });
    },
});

export const useAssetMetadataDiscovery = createApi("/asset/{asset_identity}/metadata", "get", options => assetKeys.discovery(options.path.asset_identity));

export const useAssetQuote = createApi("/asset/{asset_identity}/quote", "get", options => assetKeys.quote(options.path.asset_identity));
