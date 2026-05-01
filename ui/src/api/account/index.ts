import { queryClient } from "../client";
import { createApi, createApiMutation } from "../query";
import { components } from "../schema.gen";

export type Account = components["schemas"]["Account"];
export type WalletType = components["schemas"]["WalletType"];

export const useAccount = createApi("/acc/{account_identity}", "get", options => ["account", options.path.account_identity.toString()]);
export const useAccounts = createApi("/acc", "get", () => ["accounts"]);

export const useNextAccountId = createApi("/acc/next-id", "get", () => ["next-account-id"]);
export const useCreateAccount = createApiMutation("/acc", "post", {
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ["accounts"] });
    },
});

export const useDeleteAccount = createApiMutation("/acc/{account_identity}", "delete", {
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ["accounts"] });
    },
});

export const useUpdateAccount = createApiMutation("/acc/{account_identity}", "put", {
    onSuccess: (x) => {
        queryClient.invalidateQueries({ queryKey: ["accounts"] });
        queryClient.invalidateQueries({ queryKey: ["account", x.account_identity.toString()] });
    },
});

export const useAccountAssets = createApi("/acc/{account_identity}/assets", "get", options => ["account", options.path.account_identity.toString(), "assets"]);
export const useAddAccountAsset = createApiMutation("/acc/{account_identity}/asset/{asset_identity}", "post", {
    onSuccess: (_, x) => {
        queryClient.invalidateQueries({ queryKey: ["account", x.account_identity, "assets"] });
    },
});
export const useRemoveAccountAsset = createApiMutation("/acc/{account_identity}/asset/{asset_identity}", "delete", {
    onSuccess: (_, x) => {
        queryClient.invalidateQueries({ queryKey: ["account", x.account_identity, "assets"] });
    },
});
