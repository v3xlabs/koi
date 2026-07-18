import type { VendorFlag, VendorFlagInfo } from "../bindings.gen";
import { queryClient } from "../client";
import { createRpcMutation, createRpcQuery } from "../query";
import { rpc } from "../rpc.gen";

type Path = { path: { flag: VendorFlag; }; };

export const vendorKeys = {
    enabled: ["vendors"] as const,
    all: ["vendors-all"] as const,
};
export const useVendors = createRpcQuery<void, { vendors: VendorFlag[]; }>(
    async () => ({ vendors: await rpc.vendorListEnabled() }),
    () => vendorKeys.enabled,
);
export const useAllVendors = createRpcQuery<void, { vendors: VendorFlagInfo[]; }>(
    async () => ({ vendors: await rpc.vendorListAll() }),
    () => vendorKeys.all,
);
export const useVendorFlagEnable = createRpcMutation<Path, null>(
    options => rpc.vendorEnable(options.path.flag),
    {
        onSuccess: () => {
            void queryClient.invalidateQueries({ queryKey: vendorKeys.enabled });
        },
    },
);
export const useVendorFlagDisable = createRpcMutation<Path, null>(
    options => rpc.vendorDisable(options.path.flag),
    {
        onSuccess: () => {
            void queryClient.invalidateQueries({ queryKey: vendorKeys.enabled });
        },
    },
);

export { type VendorFlag, type VendorFlagInfo } from "../bindings.gen";
