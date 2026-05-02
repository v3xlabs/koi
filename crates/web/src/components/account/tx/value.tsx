import { Component, For, Show } from "solid-js";

import { AddressPreview } from "#/components/link/address";
import { truncateAddress } from "#/utils/address";

type JsonRecord = Record<string, unknown>;

export const isRecord = (value: unknown): value is JsonRecord => typeof value === "object" && value !== null && !Array.isArray(value);

export const titleFromKey = (key: string) => key
  .replaceAll("_", " ")
  .replaceAll(/([a-z0-9])([A-Z])/g, "$1 $2")
  .replaceAll(/\s+/g, " ")
  .trim()
  .replaceAll(/\b\w/g, letter => letter.toUpperCase());

const isHex = (value: string) => /^0x[\da-f]+$/i.test(value);

export const isAddress = (value: string) => /^0x[\da-f]{40}$/i.test(value);
const isDateString = (value: string) => /^\d{4}-\d{2}-\d{2}T/.test(value) && !Number.isNaN(Date.parse(value));

export const formatTxValue = (value: unknown): string => {
  if (value === null) return "null";

  if (value === undefined) return "-";

  if (typeof value === "boolean") return value ? "Yes" : "No";

  if (typeof value === "bigint") return value.toString();

  if (typeof value === "number") return Number.isFinite(value) ? value.toLocaleString() : value.toString();

  if (typeof value !== "string") return JSON.stringify(value);

  if (value === "") return "-";

  if (isAddress(value)) return truncateAddress(value);

  if (isDateString(value)) return new Date(value).toLocaleString();

  return value;
};

export const TxValue: Component<{ value: unknown; class?: string; network_identity?: number; }> = props => (
  <Show
    when={Array.isArray(props.value) || isRecord(props.value)}
    fallback={(
      <Show
        when={typeof props.value === "string" && isAddress(props.value)}
        fallback={(
          <code class={`break-all rounded bg-foreground/10 px-1.5 py-0.5 text-xs ${props.class ?? ""}`}>
            {formatTxValue(props.value)}
          </code>
        )}
      >
        <AddressPreview address={props.value as string} network_identity={props.network_identity} class={props.class} />
      </Show>
    )}
  >
    <TxJsonValue value={props.value} network_identity={props.network_identity} />
  </Show>
);

export const TxField: Component<{ label: string; value: unknown; network_identity?: number; }> = props => (
  <Show when={props.value !== undefined && props.value !== null}>
    <div class="min-w-0 space-y-1">
      <div class="text-[11px] font-medium uppercase tracking-wide text-muted">
        {props.label}
      </div>
      <TxValue value={props.value} network_identity={props.network_identity} />
    </div>
  </Show>
);

export const TxJsonValue: Component<{ value: unknown; network_identity?: number; }> = props => (
  <Show
    when={Array.isArray(props.value)}
    fallback={(
      <Show
        when={isRecord(props.value)}
        fallback={(
          <Show
            when={typeof props.value === "string" && isAddress(props.value)}
            fallback={<span class="break-all text-sm">{formatTxValue(props.value)}</span>}
          >
            <AddressPreview address={props.value as string} network_identity={props.network_identity} />
          </Show>
        )}
      >
        {record => (
          <div class="space-y-2 rounded border border-border/70 bg-background/40 p-2">
            <For each={Object.entries(record())}>
              {entry => (
                <div class="grid gap-1 md:grid-cols-[10rem_minmax(0,1fr)]">
                  <div class="text-xs font-medium text-muted">
                    {titleFromKey(entry[0])}
                  </div>
                  <TxJsonValue value={entry[1]} network_identity={props.network_identity} />
                </div>
              )}
            </For>
          </div>
        )}
      </Show>
    )}
  >
    {array => (
      <div class="space-y-2">
        <For each={array()}>
          {(item, index) => (
            <div class="rounded border border-border/70 bg-background/40 p-2">
              <div class="mb-1 text-[11px] font-medium uppercase tracking-wide text-muted">
                Item
{" "}
{index() + 1}
              </div>
              <TxJsonValue value={item} network_identity={props.network_identity} />
            </div>
          )}
        </For>
      </div>
    )}
  </Show>
);

export const isLongHex = (value: unknown) => typeof value === "string" && isHex(value) && value.length > 66;
