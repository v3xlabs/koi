export type Address = `0x${string}`;

export const truncateAddress = (address: Address | string | undefined) => (address ? `${address.slice(0, 6)}...${address.slice(-4)}` : "");
