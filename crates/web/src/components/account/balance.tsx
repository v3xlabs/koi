import { Component } from "solid-js";

import { useAccountBalances } from "#/api/account";
import { useDisplayCurrency } from "#/api/context";

import { AssetAmount } from "../asset/amount";

export type AccountBalanceProps = {
    account_identity: number;
};

export const AccountBalance: Component<AccountBalanceProps> = (props) => {
    const { displayCurrency } = useDisplayCurrency();
    const balanceQuery = useAccountBalances(() => ({ path: { account_identity: props.account_identity }, query: { display_currency: displayCurrency() } }));

    return (
        <AssetAmount amount={() => BigInt(balanceQuery.data?.total_quote ?? 0)} asset={displayCurrency} />
    );
};
