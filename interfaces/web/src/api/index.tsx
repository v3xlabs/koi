import { makePersisted } from "@solid-primitives/storage";
import { QueryClientProvider } from "@tanstack/solid-query";
import { Accessor, createContext, createEffect, createSignal, onCleanup, onMount, ParentComponent } from "solid-js";

import { queryClient } from "./client";

const displayCurrency = makePersisted(createSignal("fiat:usd"), { name: "display-currency" });
const privacyMode = makePersisted(createSignal(false), { name: "privacy-mode" });
const themeState = makePersisted(createSignal<"system" | "light" | "dark">("system"), { name: "theme" });

type AppContext = {
    isOnline: Accessor<boolean>;
    displayCurrency: typeof displayCurrency;
    privacyMode: typeof privacyMode;
    theme: typeof themeState;
};

export const appcontext = createContext<AppContext>({
    isOnline: () => false,
    displayCurrency,
    privacyMode,
    theme: themeState,
});

export const AppProvider: ParentComponent = (props) => {
    console.log("app provider!");
    const [isOnline, setIsOnline] = createSignal(globalThis.navigator.onLine);

    const handleOnline = () => {
        setIsOnline(globalThis.navigator.onLine);
    };

    onMount(() => {
        globalThis.addEventListener("online", handleOnline);
        globalThis.addEventListener("offline", handleOnline);
    });

    onCleanup(() => {
        globalThis.removeEventListener("online", handleOnline);
        globalThis.removeEventListener("offline", handleOnline);
    });

    createEffect(() => {
        const t = themeState[0]();
        const mq = globalThis.matchMedia("(prefers-color-scheme: dark)");

        const apply = () => {
            const resolved = t === "system" ? (mq.matches ? "dark" : "light") : t;

            document.documentElement.dataset.theme = resolved;
            const meta = document.querySelector("meta[name='theme-color']");

            if (meta) meta.setAttribute("content", resolved === "dark" ? "#2e2e2e" : "#f6f6f6");
        };

        apply();

        if (t === "system") {
            const handler = () => apply();

            mq.addEventListener("change", handler);
            onCleanup(() => mq.removeEventListener("change", handler));
        }
    });

    return (
        <QueryClientProvider client={queryClient}>
            <appcontext.Provider value={{ isOnline, displayCurrency, privacyMode, theme: themeState }}>
                {props.children}
            </appcontext.Provider>
        </QueryClientProvider>
    );
};
