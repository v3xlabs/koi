import { createFetch } from 'openapi-hooks';
import { Accessor, createContext, createSignal, onCleanup, onMount, ParentComponent } from 'solid-js';
import { createStore } from 'solid-js/store';

export const api = createFetch({
    baseUrl: 'http://localhost:3000',
    headers: {
        'Content-Type': 'application/json',
    },
    onError: (error) => {
        console.error(error);
    },
});

type AppContext = {
    isOnline: Accessor<boolean>;
}

export const appcontext = createContext<AppContext>({
    isOnline: () => false,
});

export const AppProvider: ParentComponent = (props) => {
    const [isOnline, setIsOnline] = createSignal(window.navigator.onLine);

    const handleOnline = () => {
        setIsOnline(window.navigator.onLine);
    };

    onMount(() => {
        window.addEventListener('online', handleOnline);
        window.addEventListener('offline', handleOnline);
    });

    onCleanup(() => {
        window.removeEventListener('online', handleOnline);
        window.removeEventListener('offline', handleOnline);
    });

    return (
        <appcontext.Provider value={{ isOnline }}>
            {props.children}
        </appcontext.Provider>
    )
}
