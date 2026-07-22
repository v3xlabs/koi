import type { Tx } from "../bindings.gen";
import { createRpcQuery, requireOptions } from "../query";
import { rpc } from "../rpc.gen";

export type AccountTx = Tx;
type Options = { path: { account_identity: number; }; };
export const accountTxKeys = {
    all: (identity: number | string) => ["account-tx", identity.toString()] as const,
    pending: (identity: number | string) => ["account-tx", identity.toString(), "pending"] as const,
};
export const useAccountTxAll = createRpcQuery<Options, { transactions: Tx[]; }>(async options => ({ transactions: await rpc.accountTransactionList({ account_identity: requireOptions(options).path.account_identity }) }), options => accountTxKeys.all(requireOptions(options).path.account_identity));
export const useAccountTxPending = createRpcQuery<Options, { transactions: Tx[]; }>(async options => ({ transactions: await rpc.accountTransactionPending({ account_identity: requireOptions(options).path.account_identity }) }), options => accountTxKeys.pending(requireOptions(options).path.account_identity));
