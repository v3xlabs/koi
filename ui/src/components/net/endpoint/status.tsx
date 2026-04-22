import { Accessor, Component } from "solid-js";

export const NetworkEndpointStatus: Component<{ status: Accessor<string>; }> = ({ status }) => (
    <div classList={{
        "size-2 rounded-full": true,
        "bg-[#00FF00]": status() === "Alive",
        "bg-[#FF0000]": status() === "Dead",
        "bg-[#808080]": status() === "Disabled",
    }}
    />
);
