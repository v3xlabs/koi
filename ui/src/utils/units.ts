/*
precision is how many digits after the decimal point to show
decimals is how much the value is scaled by (so 6 decimals for fiat:usd, where 123450000 is 123,45 dollars)

1234560000, decimals = 6, precision = 2
1.234,56 in long mode
1.23 k in short mode (where k = *1000, m = *1000000, b = *1000000000)
*/
export const formatUnits = (value: bigint, decimals: number, precision: number = 2, mode: "long" | "short" = "long") => {
    const result = (Number(value) / (10 ** decimals)).toFixed(precision);
    const [integer, decimal] = result.split(".");

    if (mode === "short") {
        if (Number(integer) >= 1_000_000) {
            return `${(Number(integer) / 1_000_000).toFixed(2)} m`;
        }

        if (Number(integer) >= 1000) {
            return `${(Number(integer) / 1000).toFixed(2)} k`;
        }
    }

    return `${integer}.${decimal.padEnd(precision, "0")}`;
};
