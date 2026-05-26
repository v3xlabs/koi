import { queryClient } from "../client";
import { api } from "../index";
import { createApi, createApiMutation } from "../query";
import { components } from "../schema.gen";

export type Account = components["schemas"]["Account"];
export type WalletType = components["schemas"]["WalletType"];

export const accountKeys = {
    all: ["accounts"] as const,
    detail: (account_identity: number | string) => ["account", account_identity.toString()] as const,
    nextId: ["next-account-id"] as const,
    assets: (account_identity: number | string) => ["account", account_identity.toString(), "assets"] as const,
    balances: (account_identity: number | string, display_currency: string) => ["account", account_identity.toString(), "balances", display_currency] as const,
};

export function accountBalanceQuery(
    account_identity: number | string,
    display_currency: string,
) {
    return {
        path: { account_identity: Number(account_identity) },
        query: {
            display_currency,
            fresh: false,
        },
    } as const;
}

export const useAccount = createApi("/acc/{account_identity}", "get", options => accountKeys.detail(options.path.account_identity));
export const useAccounts = createApi("/acc", "get", () => accountKeys.all, {
    onData: data => data.accounts.forEach(account => queryClient.setQueryData(accountKeys.detail(account.account_identity), account)),
});

export const useNextAccountId = createApi("/acc/next-id", "get", () => accountKeys.nextId);
export const useCreateAccount = createApiMutation("/acc", "post", {
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: accountKeys.all });
        queryClient.invalidateQueries({ queryKey: accountKeys.nextId });
    },
});

export const useDeleteAccount = createApiMutation("/acc/{account_identity}", "delete", {
    onSuccess: (_, variables) => {
        queryClient.invalidateQueries({ queryKey: accountKeys.all });

        if (variables.account_identity) {
            queryClient.removeQueries({ queryKey: accountKeys.detail(variables.account_identity) });
            queryClient.removeQueries({ queryKey: accountKeys.assets(variables.account_identity) });
        }
    },
});

export const useUpdateAccount = createApiMutation("/acc/{account_identity}", "put", {
    onSuccess: (account) => {
        queryClient.invalidateQueries({ queryKey: accountKeys.all });
        queryClient.invalidateQueries({ queryKey: accountKeys.detail(account.account_identity) });
    },
});

export const useAccountAssets = createApi("/acc/{account_identity}/assets", "get", options => accountKeys.assets(options.path.account_identity));
export const useAddAccountAsset = createApiMutation("/acc/{account_identity}/asset/{asset_identity}", "post", {
    onSuccess: (_, variables) => {
        queryClient.invalidateQueries({ queryKey: accountKeys.assets(variables.account_identity) });
    },
});
export const useRemoveAccountAsset = createApiMutation("/acc/{account_identity}/asset/{asset_identity}", "delete", {
    onSuccess: (_, variables) => {
        queryClient.invalidateQueries({ queryKey: accountKeys.assets(variables.account_identity) });
    },
});

export const useAccountBalances = createApi("/acc/{account_identity}/balances", "get", options => accountKeys.balances(options.path.account_identity, options.query.display_currency));

export async function refreshAccountBalances(options: {
    path: { account_identity: number | string; };
    query: { display_currency: string; };
}) {
    const response = await api("/acc/{account_identity}/balances", "get", {
        path: { account_identity: Number(options.path.account_identity) },
        query: {
            display_currency: options.query.display_currency,
            fresh: true,
        },
    });

    if (response.status !== 200) {
        throw new Error(response.status.toString());
    }

    queryClient.setQueryData(
        accountKeys.balances(options.path.account_identity, options.query.display_currency),
        response.data,
    );

    return response.data;
}
