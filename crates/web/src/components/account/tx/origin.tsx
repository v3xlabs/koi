import { Component, createMemo, Match, Show, Switch } from "solid-js";

import { narrow } from "#/utils/narrow";

import { TxField } from "./value";

type SafeOrigin = {
  url?: string | null;
  name?: string | null;
  note?: string | null;
};

type DecodedOrigin = {
  type: "json";
  value: SafeOrigin;
} | {
  type: "raw_json";
  value: string;
} | {
  type: "raw";
  value: string;
};

const KNOWN_ORIGINS = [
  {
    match: "https://apps-portal.safe.global/wallet-connect",
    name: "Safe WalletConnect",
    icon: "https://safe-transaction-assets.safe.global/chains/1/chain_logo.png",
  },
  {
    match: "https://swap.cow.fi",
    name: "CoW Swap",
    icon: "https://swap.cow.fi/apple-touch-icon.png",
  },
];

const decodeOrigin = (origin: string): DecodedOrigin => {
  if (origin.trim().startsWith("{")) {
    try {
      const value = JSON.parse(origin) as SafeOrigin;

      return { type: "json", value };
    }
    catch {
      return { type: "raw_json", value: origin };
    }
  }

  return { type: "raw", value: origin };
};

export const TxOrigin: Component<{ origin: string; }> = (props) => {
  const decoded = createMemo(() => decodeOrigin(props.origin));
  const originUrl = createMemo(() => {
    const x = decoded();

    if (x.type === "json") {
      return x.value.url;
    }

    if (x.type === "raw_json") {
      return x.value;
    }

    return undefined;
  });
  const knownOrigin = createMemo(() => KNOWN_ORIGINS.find(origin => originUrl()?.includes(origin.match)));

  return (
    <div class="space-y-3 rounded border border-border/70 bg-background/40 p-3">
      <Show when={knownOrigin()}>
        {known => (
          <div class="flex items-center gap-2">
            <img src={known().icon} alt={known().name} class="size-5 rounded" />
            <div>
              <div class="text-sm font-medium">{known().name}</div>
            </div>
          </div>
        )}
      </Show>
      <Switch fallback={<div class="text-sm text-muted">No origin details</div>}>
        <Match when={narrow(decoded, x => x.type === "json")}>
          {
            json => (
              <div class="grid gap-3 md:grid-cols-2">
                <TxField label="Name" value={json().value.name} />
                <TxField label="Url" value={json().value.url} />
                <TxField label="Note" value={json().value.note} />
              </div>
            )
          }
        </Match>
        <Match when={decoded().type === "raw_json" || decoded().type === "raw"}>
          <Show when={decoded().value}>
            {value => <TxField label="Raw origin" value={value()} />}
          </Show>
        </Match>
      </Switch>
    </div>
  );
};
