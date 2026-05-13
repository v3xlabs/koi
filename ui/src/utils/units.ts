/*
precision is how many digits after the decimal point to show
decimals is how much the value is scaled by (so 6 decimals for fiat:usd, where 123450000 is 123.45 dollars)

1234560000, decimals = 6, precision = 2
1,234.56 in long mode
1,23K in short mode (where K = *1000, M = *1000000, B = *1000000000)
*/
export type FormattedAmountParts = {
    prefix: string;
    integer: string;
    decimal: string;
    fraction: string;
    suffix: string;
};

type AmountFormatOptions = {
    decimals: number;
    precision?: number;
    locale?: Intl.LocalesArgument;
    style?: "decimal" | "currency";
    currency?: string;
    notation?: "standard" | "compact";
};

const withDefaults = (options: AmountFormatOptions) => ({
    precision: options.precision ?? 2,
    locale: options.locale ?? ("en-US" as Intl.LocalesArgument),
    style: options.style ?? ("decimal" as const),
    currency: options.currency?.replace("fiat:", "").toUpperCase() ?? "USD",
    notation: options.notation ?? ("standard" as const),
    decimals: options.decimals,
});

const numberFormat = (options: ReturnType<typeof withDefaults>) =>
    new Intl.NumberFormat(options.locale as Intl.LocalesArgument, {
        style: options.style,
        currency: options.style === "currency" ? options.currency : undefined,
        notation: options.notation,
        minimumFractionDigits: options.precision,
        maximumFractionDigits: options.precision,
    });

const roundsToZero = (n: number, precision: number): boolean =>
    n !== 0 && Math.abs(n) < 10 ** -precision / 2;

export const formatAmount = (
    value: bigint,
    input: AmountFormatOptions,
): string => {
    const options = withDefaults(input);
    const n = Number(value) * 10 ** -(options.decimals);
    const formatter = numberFormat(options);
    const threshold = 10 ** -options.precision;

    if (roundsToZero(n, options.precision)) {
        return `${n < 0 ? ">" : "<"}${formatter.format(n < 0 ? -threshold : threshold)}`;
    }

    return formatter.format(n);
};

export const formatAmountParts = (
    value: bigint,
    input: AmountFormatOptions,
): FormattedAmountParts => {
    const options = withDefaults(input);
    const n = Number(value) * 10 ** -(options.decimals);
    const formatter = numberFormat(options);
    const threshold = 10 ** -options.precision;

    const underflow = roundsToZero(n, options.precision);
    const underflowValue = n < 0 ? -threshold : threshold;
    const parts = formatter.formatToParts(
        underflow ? underflowValue : n,
    );

    let prefix = underflow ? (n < 0 ? ">" : "<") : "";
    let integer = "";
    let decimal = "";
    let fraction = "";
    let suffix = "";
    let seenInteger = false;

    for (const part of parts) {
        switch (part.type) {
            case "integer":
            case "group": {
                integer += part.value;
                seenInteger = true;
                break;
            }
            case "decimal": {
                decimal = part.value;
                break;
            }
            case "fraction": {
                fraction = part.value;
                break;
            }
            default: {
                 if (seenInteger) {
                    suffix += part.value;
                 }
                 else {
                    prefix += part.value;
                 }

                 break;
            }
        }
    }

    return { prefix, integer, decimal, fraction, suffix };
};

export const percentage = (part: bigint, total: bigint, decimals: number = 2) => {
    if (total === 0n) throw new Error("Cannot divide by zero");

    const scale = 10n ** BigInt(decimals);
    const value = (part * 100n * scale) / total;

    const whole = value / scale;
    const fraction = value % scale;

    return `${whole}.${fraction.toString().padStart(decimals, "0")}%`;
};

export const percentNumber = (part: bigint, total: bigint, decimals: number = 2) => {
    if (total === 0n) return 0;

    const scale = 10n ** BigInt(decimals);
    const scaled = (part * 100n * scale) / total;

    return Number(scaled) / Number(scale);
};
