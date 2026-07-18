import { z } from "zod";

import type { RpcErrorObject } from "./bindings.gen";
import { rpcErrorObjectSchema } from "./bindings.zod.gen";

const responseSchema = z.union([
    z.object({ jsonrpc: z.literal("2.0"), ["id"]: z.union([z.number(), z.string(), z.null()]), result: z.unknown() }),
    z.object({ jsonrpc: z.literal("2.0"), ["id"]: z.union([z.number(), z.string(), z.null()]), error: rpcErrorObjectSchema }),
]);

type PendingCall = {
    resolve: (value: unknown) => void;
    reject: (reason: Error) => void;
};

export class RpcClientError extends Error {
    readonly rpc: RpcErrorObject;

    constructor(rpc: RpcErrorObject) {
        super(rpc.data?.message ?? rpc.message);
        this.name = "RpcClientError";
        this.rpc = rpc;
    }
}

export type RpcBatchCall<T> = {
    method: string;
    params: Record<string, unknown>;
    parse: (value: unknown) => T;
};

export type RpcTransport = {
    call: <T>(method: string, params: Record<string, unknown>, parse: (value: unknown) => T) => Promise<T>;
    batch: <T>(calls: readonly RpcBatchCall<T>[]) => Promise<T[]>;
    close: () => void;
};

export const createRpcClient = (): RpcTransport => {
    let socket: WebSocket | undefined;
    let connecting: Promise<WebSocket> | undefined;
    let nextId = 1;
    let reconnectAttempt = 0;
    let reconnectTimer: ReturnType<typeof setTimeout> | undefined;
    let closed = false;
    const pending = new Map<number, PendingCall>();

    const rejectPending = (message: string) => {
        for (const call of pending.values()) call.reject(new Error(message));
        pending.clear();
    };

    const scheduleReconnect = () => {
        if (closed || reconnectTimer !== undefined) return;

        const delay = Math.min(5000, 250 * (2 ** reconnectAttempt));

        reconnectAttempt += 1;
        reconnectTimer = setTimeout(() => {
            reconnectTimer = undefined;
            void ensureConnected().catch(() => scheduleReconnect());
        }, delay);
    };

    const handleMessage = (event: MessageEvent<unknown>) => {
        if (typeof event.data !== "string") return;

        let decoded: unknown;

        try {
            decoded = JSON.parse(event.data);
        }
        catch {
            rejectPending("daemon returned malformed JSON");

            return;
        }

        const values = Array.isArray(decoded) ? decoded : [decoded];

        for (const value of values) {
            const response = responseSchema.safeParse(value);

            if (!response.success || typeof response.data["id"] !== "number") continue;

            const requestIdentity = response.data["id"];
            const call = pending.get(requestIdentity);

            if (!call) continue;

            pending.delete(requestIdentity);

            if ("error" in response.data) {
                call.reject(new RpcClientError(response.data.error));
                continue;
            }

            try {
                call.resolve(response.data.result);
            }
            catch (error) {
                call.reject(error instanceof Error ? error : new Error("invalid RPC result"));
            }
        }
    };

    const openSocket = async () => {
        const bootstrap = await fetch("/bootstrap", { cache: "no-store" });

        if (!bootstrap.ok) throw new Error(`daemon bootstrap failed (${bootstrap.status})`);

        const scheme = location.protocol === "https:" ? "wss:" : "ws:";
        const webSocket = new WebSocket(`${scheme}//${location.host}/rpc`);

        return await new Promise<WebSocket>((resolve, reject) => {
            webSocket.addEventListener("open", () => {
                reconnectAttempt = 0;
                resolve(webSocket);
            }, { once: true });
            webSocket.addEventListener("error", () => reject(new Error("could not connect to the Koi daemon")), { once: true });
        });
    };

    const ensureConnected = async (): Promise<WebSocket> => {
        if (closed) throw new Error("RPC client is closed");

        if (socket?.readyState === WebSocket.OPEN) return socket;

        if (connecting) return connecting;

        connecting = openSocket();

        try {
            const connected = await connecting;

            socket = connected;
            connected.addEventListener("message", handleMessage);
            connected.addEventListener("close", () => {
                if (socket === connected) socket = undefined;

                rejectPending("daemon connection closed");
                scheduleReconnect();
            });

            return connected;
        }
        finally {
            connecting = undefined;
        }
    };

    const send = async <T>(method: string, params: Record<string, unknown>, parse: (value: unknown) => T): Promise<T> => {
        const connected = await ensureConnected();
        const requestIdentity = nextId;

        nextId += 1;

        return await new Promise<T>((resolve, reject) => {
            pending.set(requestIdentity, {
                resolve: (value) => {
                    try {
                        resolve(parse(value));
                    }
                    catch (error) {
                        reject(error instanceof Error ? error : new Error("invalid RPC result"));
                    }
                },
                reject,
            });
            connected.send(JSON.stringify({ jsonrpc: "2.0", ["id"]: requestIdentity, method, params }));
        });
    };

    return {
        call: send,
        batch: async <T>(calls: readonly RpcBatchCall<T>[]) => {
            if (calls.length === 0) throw new Error("RPC batches must not be empty");

            if (calls.length > 128) throw new Error("RPC batches must not exceed 128 calls");

            const connected = await ensureConnected();
            const requests: { jsonrpc: "2.0"; ["id"]: number; method: string; params: Record<string, unknown>; }[] = [];
            const promises = calls.map(call => new Promise<T>((resolve, reject) => {
                const requestIdentity = nextId;

                nextId += 1;
                pending.set(requestIdentity, {
                    resolve: (value) => {
                        try {
                            resolve(call.parse(value));
                        }
                        catch (error) {
                            reject(error instanceof Error ? error : new Error("invalid RPC result"));
                        }
                    },
                    reject,
                });
                requests.push({ jsonrpc: "2.0", ["id"]: requestIdentity, method: call.method, params: call.params });
            }));

            connected.send(JSON.stringify(requests));

            return await Promise.all(promises);
        },
        close: () => {
            closed = true;

            if (reconnectTimer !== undefined) clearTimeout(reconnectTimer);

            socket?.close();
            rejectPending("RPC client closed");
        },
    };
};
