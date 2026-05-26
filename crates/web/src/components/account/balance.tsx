import { Component } from "solid-js";

import { accountBalanceQuery, useAccountBalances } from "#/api/account";
import { useDisplayCurrency } from "#/api/context";

import { AssetAmount } from "../asset/amount";

export type AccountBalanceProps = {
    account_identity: number;
};

export const AccountBalance: Component<AccountBalanceProps> = (props) => {
    const { displayCurrency } = useDisplayCurrency();
    const balanceQuery = useAccountBalances(() => accountBalanceQuery(props.account_identity, displayCurrency()));

    return (
        <AssetAmount amount={() => BigInt(balanceQuery.data?.total_quote ?? 0)} asset={displayCurrency} />
    );
};
