import { Component, createMemo, For, Show, Suspense } from "solid-js";

import { useAccountTxPending } from "#/api/account/tx";
import { Address } from "#/utils/address";

export const AccountTxPending: Component<{ account_identity: number; }> = ({ account_identity }) => {
    const pendingQuery = useAccountTxPending(() => ({ path: { account_identity } }));

    return (
        <div class="w-full h-full flex flex-col">
            <div>
                Pending transactions
            </div>
            <Suspense fallback={<div>Loading...</div>}>
                <Show when={pendingQuery.data}>
                    {data => (
                        <div class="w-full wrap-anywhere overflow-y-auto h-full">
                            <For each={data().transactions}>
                                {transaction => (
                                    <SafeWalletTxExperiment tx={transaction} />
                                )}
                            </For>
                        </div>
                    )}
                </Show>
            </Suspense>
        </div>
    );
};

// this is handwritten type, didnt look at docs :seek_no_evil:
type SafeTxGuess = {
    to: Address;
    safe: Address;
    value: `${bigint}`;
    data: string; // Bytes;
    operation: number;
    gasToken: Address;
    safeTxGas: number;
    baseGas: number;
    gasPrice: `${bigint}`;
    refundReceiver: Address;
    nonce: number;
    executionDate: string; // Date;
    submissionDate: string; // Date;
    modified: string; // Date;
    blockNumber: number;
    transactionHash: string;
    safeTxHash: string;
    proposer: Address;
    // proposedByDelegate: null;
    executor: Address;
    isExecuted: boolean;
    isSuccessful: boolean;
    ethGasPrice: `${bigint}`;
    maxFeePerGas: `${bigint}`;
    maxPriorityFeePerGas: `${bigint}`;
    gasUsed: bigint;
    fee: `${bigint}`;
    // payment: undefined;
    origin: string;
    dataDecoded: object; // todo
    confirmationsRequired: number;
    confirmations: { owner: Address; submissionDate: string; transactionHash?: string; signature?: string; signatureType: "EOA" | "APPROVED_HASH"; }[];
    trusted: boolean;
    signatures: string; // hex
    transfers: object[]; // todo
    txType: "MULTISIG_TRANSACTION" | "ETHEREUM_TRANSACTION";
};

export const SafeWalletTxExperiment: Component<{ tx: object; }> = ({ tx }) => {
    const x = createMemo(() => tx["extra"] as unknown as SafeTxGuess);

    return (
        <div class="border border-border rounded-md p-2">
            <table>
                <For each={Object.entries(x() ?? {})}>
                    {([key, value]) => (
                        <tr>
                            <td class="font-bold w-fit text-nowrap">
                                {key}
                            </td>
                            <td class="">
                                <div class="max-h-24 overflow-y-auto">
                                {JSON.stringify(value)}
                                </div>
                            </td>
                        </tr>
                    )}
                </For>
            </table>
        </div>
    );
};
