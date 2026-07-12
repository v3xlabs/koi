import { isAddress } from "viem";

export type Address = `0x${string}`;

const erc3770NetworkPrefixToChainId: Record<string, number> = {
    arb1: 42_161,
    arb: 42_161,
    aurora: 1_313_161_554,
    avax: 43_114,
    base: 8453,
    bnb: 56,
    bsc: 56,
    celo: 42_220,
    eth: 1,
    gno: 100,
    gor: 5,
    linea: 59_144,
    matic: 137,
    oeth: 10,
    op: 10,
    sep: 11_155_111,
    zksync: 324,
};

export type ParsedAddressInput = {
    address: string;
    network_identity?: number;
    prefix?: string;
};

export const parseAddressInput = (value: string): ParsedAddressInput => {
    const trimmed = value.trim();
    const match = /^([a-z0-9-]+):(0x[0-9a-fA-F]{40})$/.exec(trimmed);

    if (!match) return { address: trimmed };

    const prefix = match[1].toLowerCase();

    return {
        address: match[2],
        network_identity: erc3770NetworkPrefixToChainId[prefix],
        prefix,
    };
};

export const validateAddress = (value: string) => {
    const { address } = parseAddressInput(value);

    if (address.length === 0) return "Address is required";

    return isAddress(address) ? undefined : "Enter a valid EVM address";
};

export const truncateAddress = (address: Address | string | undefined) => (address ? `${address.slice(0, 6)}...${address.slice(-4)}` : "");

export const addressToHue = (address: Address | string) => {
    let hash = 0;

    for (const char of address.toLowerCase()) {
        hash = Math.trunc((hash * 31 + char.codePointAt(0)!) % 360);
    }

    return Math.abs(hash) % 360;
};
