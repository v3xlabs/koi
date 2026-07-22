import type { Account, AccountCreate, AccountGroup, AccountGroupCreate, AccountGroupUpdate, AccountLayout, AccountLayoutUpdate, AccountUpdate } from "../bindings.gen";
import { queryClient } from "../client";
import { createRpcMutation, createRpcQuery, requireOptions } from "../query";
import { rpc } from "../rpc.gen";

type AccountPath = { path: { account_identity: number; }; };
type AssetPath = { path: { account_identity: number; asset_identity: string; }; };
type AccountBalanceOptions = AccountPath & { query: { display_currency: string; fresh?: boolean; }; };

export const accountKeys = {
    all: ["accounts"] as const,
    layout: ["account-layout"] as const,
    detail: (identity: number | string) => ["account", identity.toString()] as const,
    assets: (identity: number | string) => ["account", identity.toString(), "assets"] as const,
    assetBalance: (account: number | string, asset: string, currency: string) => ["account", account.toString(), "asset", asset, "balance", currency] as const,
    balances: (account: number | string, currency: string) => ["account", account.toString(), "balances", currency] as const,
};

export const accountBalanceQuery = (account_identity: number | string, display_currency: string): AccountBalanceOptions => ({
    path: { account_identity: Number(account_identity) },
    query: { display_currency, fresh: false },
});

const invalidateAccountLists = () => {
    void queryClient.invalidateQueries({ queryKey: accountKeys.all });
    void queryClient.invalidateQueries({ queryKey: accountKeys.layout });
};

export const useAccount = createRpcQuery<AccountPath, Account>(
    options => rpc.accountGet({ account_identity: requireOptions(options).path.account_identity }),
    options => accountKeys.detail(requireOptions(options).path.account_identity),
);
export const useAccounts = createRpcQuery<void, { accounts: Account[]; }>(
    async () => ({ accounts: await rpc.accountList() }),
    () => accountKeys.all,
    {
        onData: data => data.accounts.forEach(account => queryClient.setQueryData(accountKeys.detail(account.account_identity), account)),
    },
);
export const useAccountLayout = createRpcQuery<void, AccountLayout>(
    () => rpc.accountLayoutGet(),
    () => accountKeys.layout,
    {
        onData: data => data.accounts.forEach(account => queryClient.setQueryData(accountKeys.detail(account.account_identity), account)),
    },
);
export const useCreateAccount = createRpcMutation<{ data: AccountCreate; }, Account>(
    options => rpc.accountCreate({ input: options.data }),
    {
        onSuccess: () => {
            invalidateAccountLists();
        },
    },
);
export const useDeleteAccount = createRpcMutation<AccountPath, null>(
    options => rpc.accountDelete({ account_identity: options.path.account_identity }),
    {
        onSuccess: (_, options) => {
            invalidateAccountLists();
            queryClient.removeQueries({ queryKey: accountKeys.detail(options.path.account_identity) });
            queryClient.removeQueries({ queryKey: accountKeys.assets(options.path.account_identity) });
        },
    },
);
export const useUpdateAccount = createRpcMutation<AccountPath & { data: AccountUpdate; }, Account>(
    options => rpc.accountUpdate({ account_identity: options.path.account_identity, input: options.data }),
    {
        onSuccess: (account) => {
            invalidateAccountLists();
            void queryClient.invalidateQueries({ queryKey: accountKeys.detail(account.account_identity) });
        },
    },
);
export const useUpdateAccountLayout = createRpcMutation<{ data: AccountLayoutUpdate; }, AccountLayout>(
    options => rpc.accountLayoutUpdate({ input: options.data }),
    {
        onSuccess: (layout) => {
            queryClient.setQueryData(accountKeys.layout, layout);
            invalidateAccountLists();
        },
    },
);
export const useCreateAccountGroup = createRpcMutation<{ data: AccountGroupCreate; }, AccountGroup>(
    options => rpc.accountGroupCreate({ input: options.data }),
    { onSuccess: invalidateAccountLists },
);
export const useUpdateAccountGroup = createRpcMutation<{ path: { group_identity: number; }; data: AccountGroupUpdate; }, AccountGroup>(
    options => rpc.accountGroupUpdate({ group_identity: options.path.group_identity, input: options.data }),
    { onSuccess: invalidateAccountLists },
);
export const useDeleteAccountGroup = createRpcMutation<{ path: { group_identity: number; }; }, null>(
    options => rpc.accountGroupDelete({ group_identity: options.path.group_identity }),
    { onSuccess: invalidateAccountLists },
);

export const useGenerateMnemonic = createRpcQuery<void, { mnemonic: string; }>(
    async () => ({ mnemonic: await rpc.accountMnemonicGenerate() }),
    () => ["generate-mnemonic"],
);
export const useDefaultDerivationPath = createRpcQuery<void, { path: string; }>(
    async () => ({ path: await rpc.accountDerivationDefaultPath() }),
    () => ["default-derivation-path"],
);
export const useDeriveFromMnemonic = createRpcMutation<
    { data: { mnemonic: string; paths: string[]; }; },
    { results: { path: string; address: string; }[]; }
>(async options => ({ results: await rpc.accountDerivationFromMnemonic({ input: options.data }) }));
export const useDeriveFromPrivateKey = createRpcMutation<
    { data: { private_key: string; }; },
    { address: string; }
>(async options => ({ address: await rpc.accountDerivationFromPrivateKey({ input: options.data.private_key }) }));

export const useAccountAssets = createRpcQuery<AccountPath, string[]>(
    options => rpc.accountAssetList({ account_identity: requireOptions(options).path.account_identity }),
    options => accountKeys.assets(requireOptions(options).path.account_identity),
);
export const useAccountAssetBalance = createRpcQuery<AssetPath & { query: { display_currency: string; }; }, Awaited<ReturnType<typeof rpc.accountAssetBalance>>>(
    (options) => {
        const value = requireOptions(options);

        return rpc.accountAssetBalance({ account_identity: value.path.account_identity, asset_identity: value.path.asset_identity, display_currency: value.query.display_currency });
    },
    (options) => {
        const value = requireOptions(options);

        return accountKeys.assetBalance(value.path.account_identity, value.path.asset_identity, value.query.display_currency);
    },
);
export const useAddAccountAsset = createRpcMutation<AssetPath, null>(
    options => rpc.accountAssetAdd({ account_identity: options.path.account_identity, asset_identity: options.path.asset_identity }),
    {
        onSuccess: (_, options) => {
            void queryClient.invalidateQueries({ queryKey: accountKeys.assets(options.path.account_identity) });
        },
    },
);
export const useRemoveAccountAsset = createRpcMutation<AssetPath, null>(
    options => rpc.accountAssetRemove({ account_identity: options.path.account_identity, asset_identity: options.path.asset_identity }),
    {
        onSuccess: (_, options) => {
            void queryClient.invalidateQueries({ queryKey: accountKeys.assets(options.path.account_identity) });
        },
    },
);
export const useAccountBalances = createRpcQuery<AccountBalanceOptions, Awaited<ReturnType<typeof rpc.accountBalanceList>>>(
    (options) => {
        const value = requireOptions(options);

        return rpc.accountBalanceList({ account_identity: value.path.account_identity, display_currency: value.query.display_currency, fresh: value.query.fresh ?? false });
    },
    (options) => {
        const value = requireOptions(options);

        return accountKeys.balances(value.path.account_identity, value.query.display_currency);
    },
);

export const refreshAccountBalances = async (options: AccountBalanceOptions) => {
    const data = await rpc.accountBalanceList({ account_identity: options.path.account_identity, display_currency: options.query.display_currency, fresh: true });

    queryClient.setQueryData(accountKeys.balances(options.path.account_identity, options.query.display_currency), data);

    return data;
};

export const normalizeGroupId = (group_id?: number | null): number | undefined => (
    group_id === undefined || group_id === null || group_id === 0 ? undefined : group_id
);
export const isUngroupedAccount = (account: Pick<Account, "group_id">) => normalizeGroupId(account.group_id) === undefined;
export const buildLayoutUpdate = (layout: AccountLayout): AccountLayoutUpdate => ({
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
});
export const moveAccountToGroup = (layout: AccountLayout, account_identity: number, group_id: number | undefined): AccountLayout => {
    const accounts = layout.accounts.map((account) => {
        if (account.account_identity !== account_identity) return account;

        const normalized = normalizeGroupId(group_id);
        const siblings = layout.accounts.filter(candidate => normalizeGroupId(candidate.group_id) === normalized && candidate.account_identity !== account_identity);

        return {
            ...account,
            group_id: normalized,
            display_order: siblings.length > 0 ? Math.max(...siblings.map(sibling => sibling.display_order)) + 1 : 0,
        };
    });

    return { ...layout, accounts };
};

export { type Account, type AccountCreate, type AccountGroup, type AccountGroupCreate, type AccountGroupUpdate, type AccountLayout, type AccountLayoutUpdate, type AccountUpdate, type WalletType } from "../bindings.gen";
