import { onCleanup, onMount } from "solid-js";

import { queryClient } from "./client";

type RealtimeMessage =
  | { type: "invalidate"; route: string; }
  | { type: "invalidate_all"; };

export const useAppRealtime = () => {
    onMount(() => {
        let socket: WebSocket | undefined;
        let reconnectTimer: ReturnType<typeof setTimeout> | undefined;
        let reconnectAttempt = 0;
        let closed = false;

        const connect = () => {
            socket = new WebSocket("/api/realtime");

            socket.addEventListener("open", () => {
                reconnectAttempt = 0;
            });

            socket.addEventListener("message", (event) => {
                if (typeof event.data !== "string") return;

                handleMessage(event.data);
            });

            socket.addEventListener("close", () => {
                socket = undefined;

                if (closed) return;

                const delay = Math.min(1000 * 2 ** reconnectAttempt, 30_000);

                reconnectAttempt += 1;
                reconnectTimer = setTimeout(connect, delay);
            });
        };

        connect();

        onCleanup(() => {
            closed = true;

            if (reconnectTimer) clearTimeout(reconnectTimer);

            socket?.close();
        });
    });
};

const handleMessage = (data: string) => {
    let message: RealtimeMessage;

    try {
        message = JSON.parse(data) as RealtimeMessage;
    }
    catch {
        return;
    }

    switch (message.type) {
        case "invalidate": {
            queryClient.invalidateQueries({ queryKey: routeQueryKey(message.route) });
            break;
        }
        case "invalidate_all": {
            queryClient.invalidateQueries();
            break;
        }
    }
};

const routeQueryKey = (route: string) => route.split("/").filter(Boolean);
