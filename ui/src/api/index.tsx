import { Toast, toaster } from "@kobalte/core/toast";
import { QueryClientProvider } from "@tanstack/solid-query";
import { createFetch } from "openapi-hooks";
import { FaSolidClose } from "solid-icons/fa";
import { Accessor, createContext, createSignal, onCleanup, onMount, ParentComponent } from "solid-js";

import { queryClient } from "./client";
import type { paths } from "./schema.gen";

export const api = createFetch<paths>({
    baseUrl: "http://localhost:5173/api/",
    headers: {
        "Content-Type": "application/json",
        "Authorization": "Bearer hello",
    },
    onError: (error) => {
        console.error(error);
        const id = toaster.show(props => (
            <Toast toastId={props.toastId} class="toast">
                <div class="flex justify-between items-center">
                    <div>
                        <Toast.Title class="toast__title">
                            Error #
                            {error.status}
                            {" "}
                            {error.name}
                        </Toast.Title>
                        <Toast.Description class="toast__description">
                            {error.message}
                        </Toast.Description>
                    </div>
                    <Toast.CloseButton class="toast__close-button">
                        <FaSolidClose />
                    </Toast.CloseButton>
                </div>
            </Toast>
        ));
    },
});

type AppContext = {
    isOnline: Accessor<boolean>;
};

export const appcontext = createContext<AppContext>({
    isOnline: () => false,
});

export const AppProvider: ParentComponent = (props) => {
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

    return (
        <QueryClientProvider client={queryClient}>
            <appcontext.Provider value={{ isOnline }}>
                {props.children}
            </appcontext.Provider>
        </QueryClientProvider>
    );
};
