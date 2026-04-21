import { createApi } from "../query";
import { components } from "../schema.gen";

export type Account = components["schemas"]["Account"];
export type WalletType = components["schemas"]["WalletType"];

export const useAccount = createApi("/acc/{account_id}", "get", options => ["account", options.path.account_id.toString()]);
export const useAccounts = createApi("/acc", "get", () => ["accounts"]);
