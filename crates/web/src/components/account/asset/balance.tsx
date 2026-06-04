import { Component, Show } from "solid-js";

import { useAccountAssetBalance } from "#/api/account";
import { Asset } from "#/api/asset";
import { useDisplayCurrency, usePrivacyMode } from "#/api/context";
import { privateAmount } from "#/utils/privacy";
import { formatAmount } from "#/utils/units";

type AccountAssetEntryBalanceProps = {
    account_identity: number;
    asset: Asset;
    enabled?: boolean;
};

export const AccountAssetEntryBalance: Component<AccountAssetEntryBalanceProps> = (props) => {
    const { displayCurrency } = useDisplayCurrency();
    const { privacyMode } = usePrivacyMode();

    const balanceQuery = useAccountAssetBalance(
        () => ({
            path: {
                account_identity: props.account_identity,
                asset_identity: props.asset.asset_identity,
            },
            query: {
                display_currency: displayCurrency(),
            },
        }),
        () => ({
            enabled: props.enabled ?? true,
            staleTime: 60_000,
        }),
    );

    const hasBalance = () => {
        const balance = balanceQuery.data?.balance;

        return balance !== undefined && BigInt(balance) > 0n;
    };

    return (
        <Show when={balanceQuery.isLoading || hasBalance()}>
            <div class="text-xs text-muted text-right tabular-nums shrink-0 min-w-20">
                <Show
                  when={balanceQuery.isLoading}
                  fallback={(
                    <>
                        <div>
                            {privateAmount(
                                privacyMode(),
                                formatAmount(BigInt(balanceQuery.data!.balance!), {
                                    decimals: props.asset.asset_decimals,
                                    notation: "compact",
                                }),
                            )}
                        </div>
                        <Show when={balanceQuery.data?.balance_quote}>
                            <div>
                                {privateAmount(
                                    privacyMode(),
                                    formatAmount(BigInt(balanceQuery.data!.balance_quote!), {
                                        precision: 2,
                                        decimals: 6,
                                        notation: "compact",
                                        currency: displayCurrency(),
                                    }),
                                )}
                            </div>
                        </Show>
                    </>
                )}
                >
                    <span class="inline-block animate-pulse">···</span>
                </Show>
            </div>
        </Show>
    );
};
