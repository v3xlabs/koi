import { createApi } from "../query";
import { components } from "../schema.gen";

export const accountTxKeys = {
    all: (account_identity: number | string) => ["account-tx", account_identity.toString()] as const,
    pending: (account_identity: number | string) => ["account-tx", account_identity.toString(), "pending"] as const,
};

export type AccountTx = components["schemas"]["Tx"];

export const useAccountTxAll = createApi("/acc/{account_identity}/tx", "get", options => accountTxKeys.all(options.path.account_identity));
export const useAccountTxPending = createApi("/acc/{account_identity}/tx/pending", "get", options => accountTxKeys.pending(options.path.account_identity));
