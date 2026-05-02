import { Accessor, Component, For, Show } from "solid-js";

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

const StatPill: Component<{ label: string; value: string | number; tone?: "default" | "warning"; }> = props => (
    <div classList={{
        "rounded-lg border px-3 py-2 bg-base-100/60": true,
        "border-primary/50": props.tone === "warning",
    }}
    >
        <div class="text-[10px] uppercase tracking-wide opacity-60">{props.label}</div>
        <div class="text-sm font-medium tabular-nums">{props.value}</div>
    </div>
);

const formatTime = (timestamp?: number) => {
    if (!timestamp) {
        return "never";
    }

    return new Date(timestamp).toLocaleTimeString();
};

export const NetworkEndpointRpcStats: Component<{ stats: Accessor<RpcEndpointStats>; compact?: boolean; }> = (props) => {
    const activeLabel = () => (
        props.stats().queued > 0
            ? `${props.stats().in_flight} active, ${props.stats().queued} waiting`
            : `${props.stats().in_flight} active`
    );
    const errorTone = () => (
        props.stats().total_errors > 0 ? "warning" : "default"
    );

    return (
        <div class="space-y-3 text-start">
            <div class="grid grid-cols-2 md:grid-cols-4 gap-2">
                <StatPill label="Traffic" value={`${props.stats().total_requests} calls`} />
                <StatPill label="Now" value={activeLabel()} />
                <StatPill label="Peak" value={`${props.stats().max_in_flight} in flight`} />
                <StatPill
                  label="Errors"
                  value={`${props.stats().total_errors} total`}
                  tone={errorTone()}
                />
            </div>
            <Show when={!props.compact}>
                <div class="grid grid-cols-2 md:grid-cols-4 gap-2">
                    <StatPill label="Latency" value={`${props.stats().average_duration_ms}ms avg`} />
                    <StatPill label="Rate limited" value={props.stats().total_rate_limited} tone={props.stats().total_rate_limited > 0 ? "warning" : "default"} />
                    <StatPill label="Connected" value={`${props.stats().connection_successes}/${props.stats().connection_attempts}`} />
                    <StatPill label="Last request" value={formatTime(props.stats().last_request_at)} />
                </div>
            </Show>
            <Show when={props.stats().methods.length > 0}>
                <div class="flex flex-wrap gap-1.5">
                    <For each={props.stats().methods.slice(0, props.compact ? 3 : 6)}>
                        {method => (
                            <span class="rounded-full border px-2 py-1 text-xs tabular-nums bg-base-100/60">
                                {method.errors > 0
                                    ? `${method.method} ${method.total} / ${method.errors} errors`
                                    : `${method.method} ${method.total}`}
                            </span>
                        )}
                    </For>
                </div>
            </Show>
            <Show when={!props.compact && props.stats().last_error}>
                {error => (
                    <div class="rounded-lg border border-primary/50 p-3 text-xs">
                        <div class="font-medium">Last RPC error</div>
                        <div class="opacity-75 break-words">{error()}</div>
                    </div>
                )}
            </Show>
        </div>
    );
};
