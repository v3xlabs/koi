import { createFetch } from "openapi-hooks";
import { Accessor, createContext, createSignal, onCleanup, onMount, ParentComponent } from "solid-js";

export const api = createFetch({
    baseUrl: "http://localhost:3000",
    headers: {
        "Content-Type": "application/json",
    },
    onError: (error) => {
        console.error(error);
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
        <appcontext.Provider value={{ isOnline }}>
            {props.children}
        </appcontext.Provider>
    );
};
