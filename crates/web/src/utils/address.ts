export type Address = `0x${string}`;

export const truncateAddress = (address: Address | string | undefined) => (address ? `${address.slice(0, 6)}...${address.slice(-4)}` : "");

export const addressToHue = (address: Address | string) => {
    let hash = 0;

    for (const char of address.toLowerCase()) {
        hash = Math.trunc((hash * 31 + char.codePointAt(0)!) % 360);
    }

    return Math.abs(hash) % 360;
};
