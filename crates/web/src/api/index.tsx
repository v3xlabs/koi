import { Toast, toaster } from "@kobalte/core/toast";
import { makePersisted } from "@solid-primitives/storage";
import { QueryClientProvider } from "@tanstack/solid-query";
import { createFetch } from "openapi-hooks";
import { FaSolidClose } from "solid-icons/fa";
import { Accessor, createContext, createEffect, createSignal, onCleanup, onMount, ParentComponent } from "solid-js";

import { queryClient } from "./client";
import type { paths } from "./schema.gen";

const baseUrl = location.origin + "/api/";

export const api = createFetch<paths>({
    baseUrl,
    headers: {
        "Content-Type": "application/json",
        "Authorization": "Bearer hello",
    },
    onError: async (error) => {
        // const errorData = (await error.response?.json()) as { error: string; };

        // console.error(error, errorData);

        // const id = toaster.show(props => (
        //     <Toast toastId={props.toastId} class="toast">
        //         <div class="flex justify-between items-center">
        //             <div>
        //                 <Toast.Title class="toast__title">
        //                     Error #
        //                     {error.status}
        //                     {" "}
        //                     {error.name}
        //                 </Toast.Title>
        //                 <Toast.Description class="toast__description">
        //                     {error.message}
        //                     <div>
        //                         {errorData.error}
        //                     </div>
        //                 </Toast.Description>
        //             </div>
        //             <Toast.CloseButton class="toast__close-button">
        //                 <FaSolidClose />
        //             </Toast.CloseButton>
        //         </div>
        //     </Toast>
        // ));
    },
});

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
        const mq = window.matchMedia("(prefers-color-scheme: dark)");

        const apply = () => {
            const resolved = t === "system" ? (mq.matches ? "dark" : "light") : t;
            document.documentElement.dataset.theme = resolved;
            const meta = document.querySelector('meta[name="theme-color"]');
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
