import { useContext } from "solid-js";

import { appcontext } from ".";

export const useDisplayCurrency = () => {
    const { displayCurrency: [displayCurrency, setDisplayCurrency] } = useContext(appcontext);

    return { displayCurrency, setDisplayCurrency };
};

export const usePrivacyMode = () => {
    const { privacyMode: [privacyMode, setPrivacyMode] } = useContext(appcontext);

    return { privacyMode, setPrivacyMode };
};

export const useTheme = () => {
    const { theme: [theme, setTheme] } = useContext(appcontext);

    return { theme, setTheme };
};
