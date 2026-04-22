import { connectSession, Session, SessionState } from "@openlv/session";
import { webrtc } from "@openlv/transport";
import { createMemo } from "solid-js";
import { createStore } from "solid-js/store";

export type Connection = {
    connection_id: string;
    status: SessionState;
    account_id: string;
    network_id: string;
    session: Session;
};

const connections = createStore<Connection[]>([]);

export const useConnections = () => createMemo(() => connections[0]);
export const removeConnection = (connection_id: string) => {
    connections[1](prev => prev.filter(c => c.connection_id !== connection_id));
};
export const addConnection = async (url: string, account_id: string, network_id: string) => {
    const session = await connectSession(url, async (msg) => {
        console.log("SMSG", msg);

        return msg;
    }, webrtc());

    const connection: Connection = {
        connection_id: crypto.randomUUID(),
        status: session.getState().status,
        account_id,
        network_id,
        session,
    };

    session.emitter.on("state_change", (state) => {
        console.log("STATE CHANGE", state?.status);
        connections[1](prev => prev.map(c => (c.connection_id === connection.connection_id ? { ...c, status: state?.status ?? "disconnected" } : c)));
    });

    connections[1](prev => [...prev, connection]);

    await session.connect();
};
