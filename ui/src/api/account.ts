import { Address } from "blo";

const HARDCODED_ACCOUNTS: { account_id: number; evm_address: Address; }[] = [
    {
        account_id: 1,
        evm_address: "0x225f137127d9067788314bc7fcc1f36746a3c3B5",
    },
    {
        account_id: 2,
        evm_address: "0x225f137127d9067788314bc7fcc1f36746a3c3B5",
    },
    {
        account_id: 3,
        evm_address: "0x225f137127d9067788314bc7fcc1f36746a3c3B5",
    },
];

export const useAccounts = () => HARDCODED_ACCOUNTS;

export const useAccount = (account_id: number) => HARDCODED_ACCOUNTS.find(account => account.account_id === account_id);
