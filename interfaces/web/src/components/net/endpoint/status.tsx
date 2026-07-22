import { Accessor, Component, Show } from "solid-js";

import { RpcEndpointStats } from "#/api/network";

export const NetworkEndpointStatus: Component<{ status: Accessor<string>; }> = ({ status }) => (
    <div classList={{
        "size-2 rounded-full": true,
        "bg-[#00FF00]": status() === "Alive",
        "bg-[#FF0000]": status() === "Dead",
        "bg-[#808080]": status() === "Disabled",
    }}
    />
);

const formatTime = (timestamp?: number) => {
    if (!timestamp) {
        return "never";
    }

    return new Date(timestamp).toLocaleTimeString();
};

export const NetworkEndpointRpcStats: Component<{ stats: Accessor<RpcEndpointStats>; }> = (props) => {
    const activeLabel = () => (
        props.stats().queued > 0
            ? `${props.stats().in_flight} active, ${props.stats().queued} waiting`
            : `${props.stats().in_flight} active`
    );

    return (
        <div class="space-y-4 text-start">
            <dl class="grid grid-cols-2 gap-x-6 gap-y-3 text-sm sm:grid-cols-4">
                <div>
                    <dt class="text-muted">Calls</dt>
                    <dd class="tabular-nums">{props.stats().total_requests}</dd>
                </div>
                <div>
                    <dt class="text-muted">Active</dt>
                    <dd class="tabular-nums">{activeLabel()}</dd>
                </div>
                <div>
                    <dt class="text-muted">Latency</dt>
                    <dd class="tabular-nums">
                        {props.stats().average_duration_ms}
                        {" "}
                        ms
                    </dd>
                </div>
                <div>
                    <dt class="text-muted">Errors</dt>
                    <dd class="tabular-nums">{props.stats().total_errors}</dd>
                </div>
            </dl>
            <details class="text-sm">
                <summary class="cursor-pointer text-muted hover:text-foreground">Request detail</summary>
                <dl class="mt-3 grid grid-cols-2 gap-x-6 gap-y-3 sm:grid-cols-4">
                    <div>
                        <dt class="text-muted">Peak</dt>
                        <dd class="tabular-nums">{props.stats().max_in_flight}</dd>
                    </div>
                    <div>
                        <dt class="text-muted">Rate limited</dt>
                        <dd class="tabular-nums">{props.stats().total_rate_limited}</dd>
                    </div>
                    <div>
                        <dt class="text-muted">Connected</dt>
                        <dd class="tabular-nums">
                            {props.stats().connection_successes}
                            /
                            {props.stats().connection_attempts}
                        </dd>
                    </div>
                    <div>
                        <dt class="text-muted">Last request</dt>
                        <dd>{formatTime(props.stats().last_request_at)}</dd>
                    </div>
                </dl>
            </details>
            <Show when={props.stats().last_error}>
                {error => (
                    <div class="border-l-2 border-primary pl-3 text-xs">
                        <div class="text-muted">Last error</div>
                        <div class="break-words">{error()}</div>
                    </div>
                )}
            </Show>
        </div>
    );
};
