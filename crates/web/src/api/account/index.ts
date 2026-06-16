import { queryClient } from "../client";
import { api } from "../index";
import { createApi, createApiMutation } from "../query";
import { components } from "../schema.gen";

export type Account = components["schemas"]["Account"];
export type AccountGroup = components["schemas"]["AccountGroup"];
export type AccountLayout = components["schemas"]["AccountLayout"];
export type AccountLayoutUpdate = components["schemas"]["AccountLayoutUpdate"];
export type WalletType = components["schemas"]["WalletType"];

export const accountKeys = {
    all: ["accounts"] as const,
    layout: ["account-layout"] as const,
    detail: (account_identity: number | string) => ["account", account_identity.toString()] as const,
    nextId: ["next-account-id"] as const,
    assets: (account_identity: number | string) => ["account", account_identity.toString(), "assets"] as const,
    assetBalance: (account_identity: number | string, asset_identity: string, display_currency: string) => ["account", account_identity.toString(), "asset", asset_identity, "balance", display_currency] as const,
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

const invalidateAccountLists = () => {
    queryClient.invalidateQueries({ queryKey: accountKeys.all });
    queryClient.invalidateQueries({ queryKey: accountKeys.layout });
};

export const useAccount = createApi("/acc/{account_identity}", "get", options => accountKeys.detail(options.path.account_identity));
export const useAccounts = createApi("/acc", "get", () => accountKeys.all, {
    onData: data => data.accounts.forEach(account => queryClient.setQueryData(accountKeys.detail(account.account_identity), account)),
});
export const useAccountLayout = createApi("/acc/layout", "get", () => accountKeys.layout, {
    onData: data => data.accounts.forEach(account => queryClient.setQueryData(accountKeys.detail(account.account_identity), account)),
});

export const useNextAccountId = createApi("/acc/next-id", "get", () => accountKeys.nextId);
export const useCreateAccount = createApiMutation("/acc", "post", {
    onSuccess: () => {
        invalidateAccountLists();
        queryClient.invalidateQueries({ queryKey: accountKeys.nextId });
    },
});

export const useDeleteAccount = createApiMutation("/acc/{account_identity}", "delete", {
    onSuccess: (_, variables) => {
        invalidateAccountLists();

        if (variables.account_identity) {
            queryClient.removeQueries({ queryKey: accountKeys.detail(variables.account_identity) });
            queryClient.removeQueries({ queryKey: accountKeys.assets(variables.account_identity) });
        }
    },
});

export const useUpdateAccount = createApiMutation("/acc/{account_identity}", "put", {
    onSuccess: (account) => {
        invalidateAccountLists();
        queryClient.invalidateQueries({ queryKey: accountKeys.detail(account.account_identity) });
    },
});

export const useUpdateAccountLayout = createApiMutation("/acc/layout", "put", {
    onSuccess: (layout) => {
        queryClient.setQueryData(accountKeys.layout, layout);
        invalidateAccountLists();
    },
});

export const useCreateAccountGroup = createApiMutation("/acc/groups", "post", {
    onSuccess: () => {
        invalidateAccountLists();
    },
});

export const useUpdateAccountGroup = createApiMutation("/acc/groups/{group_identity}", "put", {
    onSuccess: () => {
        invalidateAccountLists();
    },
});

export const useDeleteAccountGroup = createApiMutation("/acc/groups/{group_identity}", "delete", {
    onSuccess: () => {
        invalidateAccountLists();
    },
});

export const useGenerateMnemonic = createApi("/acc/generate/mnemonic", "get", () => ["generate-mnemonic"]);
export const useDefaultDerivationPath = createApi("/acc/derive/default-path", "get", () => ["default-derivation-path"]);
export const useDeriveFromMnemonic = createApiMutation("/acc/derive/mnemonic", "post");
export const useDeriveFromPrivateKey = createApiMutation("/acc/derive/private-key", "post");

export const useAccountAssets = createApi("/acc/{account_identity}/assets", "get", options => accountKeys.assets(options.path.account_identity));
export const useAccountAssetBalance = createApi("/acc/{account_identity}/asset/{asset_identity}/balance", "get", options => accountKeys.assetBalance(options.path.account_identity, options.path.asset_identity, options.query.display_currency));
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

export function normalizeGroupId(group_id?: number | null): number | undefined {
    if (group_id === undefined || group_id === null || group_id === 0) {
        return undefined;
    }

    return group_id;
}

export function isUngroupedAccount(account: Pick<Account, "group_id">) {
    return normalizeGroupId(account.group_id) === undefined;
}

export function buildLayoutUpdate(layout: AccountLayout): AccountLayoutUpdate {
    return {
        groups: layout.groups.map(group => ({
            group_identity: group.group_identity,
            name: group.name,
            display_order: group.display_order,
        })),
        accounts: layout.accounts.map(account => ({
            account_identity: account.account_identity,
            group_id: normalizeGroupId(account.group_id),
            display_order: account.display_order,
        })),
    };
}

export function moveAccountToGroup(
    layout: AccountLayout,
    account_identity: number,
    group_id: number | undefined,
): AccountLayout {
    const accounts = layout.accounts.map((account) => {
        if (account.account_identity !== account_identity) return account;

        const normalizedGroupId = normalizeGroupId(group_id);
        const siblings = layout.accounts.filter(
            candidate => normalizeGroupId(candidate.group_id) === normalizedGroupId
              && candidate.account_identity !== account_identity,
        );
        const display_order = siblings.length > 0
            ? Math.max(...siblings.map(sibling => sibling.display_order)) + 1
            : 0;

        return { ...account, group_id: normalizedGroupId, display_order };
    });

    return { ...layout, accounts };
}
