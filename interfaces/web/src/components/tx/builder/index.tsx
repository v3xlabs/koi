import { FaSolidBox, FaSolidCheck } from "solid-icons/fa";
import { FiArrowDown, FiCode, FiRotateCw, FiSend } from "solid-icons/fi";
import { Address } from "viem";

export type BuilderTxBase<T, D> = {
    type: T;
    data: Partial<D>;
};

export type BuilderTxRaw = BuilderTxBase<"raw", {
    to: Address;
    value: bigint;
    data: string;
}>;

export type BuilderTxSend = BuilderTxBase<"send", {
    to: Address;
    token: Address;
    value: bigint;
    data: string;
}>;

export type BuilderTxApprove = BuilderTxBase<"approve", {
    token: Address;
    spender: Address;
    value: bigint;
}>;

export type BuilderTxWrap = BuilderTxBase<"wrap", {
    token: Address;
    amount: bigint;
}>;

export type BuilderTxUnwrap = BuilderTxBase<"unwrap", {
    token: Address;
    amount: bigint;
}>;

export type BuilderTxDeposit = BuilderTxBase<"deposit", {
    vault: Address;
    amount: bigint;
}>;

export type BuilderTxWithdraw = BuilderTxBase<"withdraw", {
    vault: Address;
    amount: bigint;
}>;

export type BuilderTxSwap = BuilderTxBase<"swap", {
    tokenIn: Address;
    tokenOut: Address;
    amountIn: bigint;
    provider: string; // uniswap_v2, uniswap_v3, etc.

    amountOutMin: bigint;
    deadline: bigint;
}>;

export type BuilderTx = BuilderTxRaw
  | BuilderTxSend
  | BuilderTxApprove
  | BuilderTxWrap
  | BuilderTxUnwrap
  | BuilderTxDeposit
  | BuilderTxWithdraw
  | BuilderTxSwap;

export const TX_TYPE_META = {
    send: { name: "Send" },
    swap: { name: "Swap" },
    deposit: { name: "Deposit" },
    withdraw: { name: "Withdraw" },
    approve: { name: "Approve" },
    wrap: { name: "Wrap" },
    unwrap: { name: "Unwrap" },
    raw: { name: "Raw" },
} as const satisfies Record<BuilderTx["type"], { name: string; }>;

export const TX_PRESETS = [
    {
        type: "send",
        name: "Send",
        icon: FiSend,
        description: "Send tokens to an address",
    },
    {
        type: "swap",
        name: "Swap",
        icon: FiRotateCw,
        description: "Swap tokens between two tokens",
    },
    {
        type: "deposit",
        name: "Deposit / Withdraw",
        icon: FiArrowDown,
        description: "Deposit or withdraw from a vault",
    },
    {
        type: "approve",
        name: "Approve",
        icon: FaSolidCheck,
        description: "Approve a spender to spend tokens on your behalf",
    },
    {
        type: "wrap",
        name: "Wrap / Unwrap",
        icon: FaSolidBox,
        description: "Wrap or unwrap tokens",
    },
    {
        type: "raw",
        name: "Raw",
        icon: FiCode,
        description: "Execute a raw transaction",
    },
] as const;
