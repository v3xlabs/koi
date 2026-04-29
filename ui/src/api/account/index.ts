import { queryClient } from "../client";
import { createApi, createApiMutation } from "../query";
import { components } from "../schema.gen";

export type Account = components["schemas"]["Account"];
export type WalletType = components["schemas"]["WalletType"];

export const useAccount = createApi("/acc/{account_id}", "get", options => ["account", options.path.account_id.toString()]);
export const useAccounts = createApi("/acc", "get", () => ["accounts"]);

export const useNextAccountId = createApi("/acc/next-id", "get", () => ["next-account-id"]);
export const useCreateAccount = createApiMutation("/acc", "post", {
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ["accounts"] });
    },
});

export const useDeleteAccount = createApiMutation("/acc/{account_id}", "delete", {
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ["accounts"] });
    },
});

export const useUpdateAccount = createApiMutation("/acc/{account_id}", "put", {
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ["accounts"] });
    },
});
