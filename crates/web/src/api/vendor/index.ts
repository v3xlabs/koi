import { queryClient } from "../client";
import { createApi, createApiMutation } from "../query";
import { components } from "../schema.gen";

export const vendorKeys = {
    enabled: ["vendors"] as const,
    all: ["vendors-all"] as const,
};

export type VendorFlagInfo = components["schemas"]["VendorFlagInfo"];
export type VendorFlag = components["schemas"]["VendorFlag"];

export const useVendors = createApi("/vendor", "get", () => vendorKeys.enabled, {});
export const useAllVendors = createApi("/vendor/all", "get", () => vendorKeys.all, {});

export const useVendorFlagEnable = createApiMutation("/vendor/{flag}", "post", {
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: vendorKeys.enabled });
    },
});

export const useVendorFlagDisable = createApiMutation("/vendor/{flag}", "delete", {
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: vendorKeys.enabled });
    },
});
