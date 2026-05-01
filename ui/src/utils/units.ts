export const formatUnits = (value: bigint, decimals: number, precision: number = 2) => {
    const result = (Number(value) / 10 ** decimals).toFixed(precision);
    const [integer, decimal] = result.split(".");

    return `${integer}.${decimal.padEnd(precision, "0")}`;
};
