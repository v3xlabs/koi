export const PRIVATE_AMOUNT = "••••••";

export const privateAmountTitle = (privacyMode: boolean, title?: string) => privacyMode ? undefined : title;

export const privateAmount = (privacyMode: boolean, value: string) => privacyMode ? PRIVATE_AMOUNT : value;
