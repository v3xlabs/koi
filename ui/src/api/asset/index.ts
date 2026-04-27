import { queryClient } from "../client";
import { createApi, createApiMutation } from "../query";
import { components } from "../schema.gen";

export type Asset = components["schemas"]["Asset"];
export type AssetUpdate = components["schemas"]["AssetUpdate"];

export const useAssets = createApi("/asset", "get", () => ["assets"]);
export const useAsset = createApi("/asset/{asset_id}", "get", options => ["assets", options.path.asset_id.toString()]);
export const useCreateAsset = createApiMutation("/asset", "post", {
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ["assets"] });
    },
});
export const useUpdateAsset = createApiMutation("/asset/{asset_id}", "put", {
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ["assets"] });
    },
});
export const useDeleteAsset = createApiMutation("/asset/{asset_id}", "delete", {
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ["assets"] });
    },
});
