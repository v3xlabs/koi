import { Tooltip } from "@kobalte/core/tooltip";
import { FiGitBranch } from "solid-icons/fi";
import { Component, For, Match, Show, Switch } from "solid-js";

import type { Decoded, DecodedCall, DecodedParam } from "#/api/bindings.gen";
import { usePrivacyMode } from "#/api/context";
import { AssetAmount } from "#/components/asset/amount";
import { AssetPreview } from "#/components/asset/preview";
import { AddressPreview } from "#/components/link/address";
import { privateAmount } from "#/utils/privacy";

import { TxField, TxJsonValue, TxValue } from "./value";

const callTitle = (call: DecodedCall) => {
  const decoded = call.decoded;

  if (decoded.kind === "verified") return decoded.function;

  if (decoded.kind === "signature_fallback") return `Unknown ${decoded.selector}`;

  return "Raw call";
};

const operationLabel = (operation: string | undefined) => operation ?? "call";
const subcalls = (call: DecodedCall) => call.subcalls ?? [];

const paramValue = (params: DecodedParam[], name: string) => params.find(param => param.name === name)?.value;
const paramAddress = (params: DecodedParam[], name: string) => {
  const value = paramValue(params, name);

  return typeof value === "string" ? value : undefined;
};

const paramText = (params: DecodedParam[], name: string) => {
  const value = paramValue(params, name);

  if (typeof value === "string" || typeof value === "number" || typeof value === "boolean") return String(value);

  return undefined;
};

const erc20AssetIdentity = (networkIdentity: number, address: string | undefined) => (address ? `erc20:${networkIdentity}:${address}` : undefined);
const bigintAmount = (value: string | undefined) => (value && /^\d+$/.test(value) ? BigInt(value) : undefined);

const handledPreview = (call: DecodedCall, networkIdentity: number) => {
  const decoded = call.decoded;

  if (decoded.kind !== "verified") return undefined;

  const params = decoded.params;

  switch (decoded.function) {
    case "approve": {
      return {
        action: "Approve",
        primary: paramAddress(params, "spender"),
        primaryLabel: "spender",
        amount: paramText(params, "value") ?? paramText(params, "amount"),
        asset: erc20AssetIdentity(networkIdentity, call.to),
      };
    }
    case "transfer": {
      return {
        action: "Transfer",
        primary: paramAddress(params, "to") ?? paramAddress(params, "_to"),
        primaryLabel: "to",
        amount: paramText(params, "value") ?? paramText(params, "_value") ?? paramText(params, "amount"),
        asset: erc20AssetIdentity(networkIdentity, call.to),
      };
    }
    case "transferFrom": {
      return {
        action: "Transfer from",
        primary: paramAddress(params, "from"),
        primaryLabel: "from",
        secondary: paramAddress(params, "to"),
        secondaryLabel: "to",
        amount: paramText(params, "value") ?? paramText(params, "amount"),
        asset: erc20AssetIdentity(networkIdentity, call.to),
      };
    }
    case "erc20Transfer": {
      return {
        action: "ERC20 transfer",
        primary: paramAddress(params, "token"),
        primaryLabel: "token",
        secondary: paramAddress(params, "receiver"),
        secondaryLabel: "receiver",
        amount: paramText(params, "amount"),
        asset: erc20AssetIdentity(networkIdentity, paramAddress(params, "token")),
      };
    }
    case "erc20TransferFrom": {
      return {
        action: "ERC20 transfer from",
        primary: paramAddress(params, "token"),
        primaryLabel: "token",
        secondary: paramAddress(params, "receiver"),
        secondaryLabel: "receiver",
        amount: paramText(params, "amount"),
        asset: erc20AssetIdentity(networkIdentity, paramAddress(params, "token")),
      };
    }
    case "erc4626Deposit": {
      return {
        action: "ERC4626 deposit",
        primary: paramAddress(params, "vault"),
        primaryLabel: "vault",
        secondary: paramAddress(params, "receiver"),
        secondaryLabel: "receiver",
        amount: paramText(params, "assets"),
        // asset: erc20AssetIdentity(networkIdentity, paramAddress(params, "vault")),
      };
    }
    case "erc4626Redeem": {
      return {
        action: "ERC4626 redeem",
        primary: paramAddress(params, "vault"),
        primaryLabel: "vault",
        secondary: paramAddress(params, "receiver"),
        secondaryLabel: "receiver",
        amount: paramText(params, "shares"),
        asset: erc20AssetIdentity(networkIdentity, paramAddress(params, "vault")),
      };
    }
    default: {
      return undefined;
    }
  }
};

const shouldShowParams = (call: DecodedCall) => call.decoded.kind === "verified" && subcalls(call).length === 0 && !handledPreview(call, 1);

const TargetPreview: Component<{ decoded: Decoded; to: string; network_identity: number; }> = props => (
  <div class="flex min-w-0 flex-wrap items-center justify-end gap-1.5 text-right">
    <Show when={props.decoded.kind === "verified" ? props.decoded.contract.verified_name : undefined}>
      {name => <span class="text-sm font-medium">{name()}</span>}
    </Show>
    <AddressPreview address={props.to} network_identity={props.network_identity} />
    <Show when={props.decoded.kind === "verified" ? props.decoded.contract.proxy : undefined}>
      {proxy => (
        <>
          <Tooltip placement="top">
            <Tooltip.Trigger class="inline-flex items-center gap-1 rounded bg-primary/10 px-1.5 py-0.5 text-xs text-primary">
              <FiGitBranch class="size-3" />
              <span>proxy</span>
            </Tooltip.Trigger>
            <Tooltip.Portal>
              <Tooltip.Content class="bg-surface-alt text-secondary-foreground max-w-xs rounded-md border border-border p-2 text-xs">
                <Tooltip.Arrow />
                Proxy
                <Show when={proxy().proxy_type}>
                  {type => (
                    <>
                      :
                      {type()}
                    </>
                  )}
                </Show>
              </Tooltip.Content>
            </Tooltip.Portal>
          </Tooltip>
          <span class="text-xs text-muted">impl</span>
          <Show when={proxy().implementation_name}>
            {name => <span class="text-xs text-muted">{name()}</span>}
          </Show>
          <AddressPreview address={proxy().implementation} network_identity={props.network_identity} />
        </>
      )}
    </Show>
  </div>
);

const OperationBadge: Component<{ operation: string | undefined; }> = props => (
  <code class="rounded bg-foreground/10 px-1.5 py-0.5 text-xs text-muted">
    {operationLabel(props.operation)}
  </code>
);

const MaybeAsset: Component<{ asset_identity: string | undefined; fallback: string | undefined; network_identity: number; }> = props => (
  <Show
    when={props.asset_identity}
    fallback={(
      <Show when={props.fallback}>
        {address => <AddressPreview address={address()} network_identity={props.network_identity} />}
      </Show>
    )}
  >
    {asset => <AssetPreview asset_identity={asset()} />}
  </Show>
);

const MaybeAmount: Component<{ amount: string | undefined; asset_identity: string | undefined; }> = (props) => {
  const { privacyMode } = usePrivacyMode();

  return (
    <Show
      when={props.asset_identity && bigintAmount(props.amount) !== undefined ? props.asset_identity : undefined}
      fallback={(
        <Show when={props.amount}>
          {amount => <code class="rounded bg-foreground/10 px-1.5 py-0.5 text-xs">{privateAmount(privacyMode(), amount())}</code>}
        </Show>
      )}
    >
      {asset => <AssetAmount amount={() => bigintAmount(props.amount)!} asset={() => asset()} />}
    </Show>
  );
};

const CallSummary: Component<{ call: DecodedCall; network_identity: number; }> = props => (
  <Show when={handledPreview(props.call, props.network_identity)}>
    {preview => (
      <div class="flex min-w-0 flex-wrap items-center gap-2 text-sm">
        <span class="font-medium">{preview().action}</span>
        <MaybeAmount amount={preview().amount} asset_identity={preview().asset} />
        <Show when={preview().primary}>
          {address => (
            <>
              <span class="text-xs text-muted">{preview().primaryLabel}</span>
              <MaybeAsset asset_identity={preview().primaryLabel === "token" || preview().primaryLabel === "vault" ? erc20AssetIdentity(props.network_identity, address()) : undefined} fallback={address()} network_identity={props.network_identity} />
            </>
          )}
        </Show>
        <Show when={preview().secondary}>
          {address => (
            <>
              <span class="text-xs text-muted">{preview().secondaryLabel}</span>
              <AddressPreview address={address()} network_identity={props.network_identity} />
            </>
          )}
        </Show>
      </div>
    )}
  </Show>
);

const CallTitle: Component<{ call: DecodedCall; network_identity: number; }> = props => (
  <Show
    when={handledPreview(props.call, props.network_identity)}
    fallback={<div class="truncate font-medium">{callTitle(props.call)}</div>}
  >
    <CallSummary call={props.call} network_identity={props.network_identity} />
  </Show>
);

const ParamValue: Component<{ param: DecodedParam; network_identity: number; }> = props => (
  <Show
    when={Array.isArray(props.param.value) || (typeof props.param.value === "object" && props.param.value !== null)}
    fallback={<TxValue value={props.param.value} network_identity={props.network_identity} />}
  >
    <TxJsonValue value={props.param.value} network_identity={props.network_identity} />
  </Show>
);

const DecodedParams: Component<{ params: DecodedParam[]; network_identity: number; }> = props => (
  <Show when={props.params.length > 0}>
    <div class="space-y-2">
      <div class="text-xs font-medium uppercase tracking-wide text-muted">Parameters</div>
      <div class="space-y-2">
        <For each={props.params}>
          {param => (
            <div class="rounded border border-border/70 bg-background/40 p-3">
              <div class="mb-2 flex flex-wrap items-center gap-2">
                <span class="font-medium">{param.name || "Unnamed"}</span>
                <code class="rounded bg-foreground/10 px-1.5 py-0.5 text-xs text-muted">{param.ty}</code>
              </div>
              <ParamValue param={param} network_identity={props.network_identity} />
            </div>
          )}
        </For>
      </div>
    </div>
  </Show>
);

const DecodedKind: Component<{ call: DecodedCall; network_identity: number; }> = props => (
  <Switch>
    <Match when={props.call.decoded.kind === "verified"}>
      <div class="space-y-2">
        <div class="grid gap-2 md:grid-cols-2" classList={{ hidden: props.call.decoded.kind !== "verified" || !props.call.decoded.signature || Boolean(handledPreview(props.call, props.network_identity)) || subcalls(props.call).length > 0 }}>
          <TxField label="Signature" value={props.call.decoded.kind === "verified" ? props.call.decoded.signature : undefined} />
        </div>
        <Show when={shouldShowParams(props.call) && props.call.decoded.kind === "verified" ? props.call.decoded.params : undefined}>
          {params => <DecodedParams params={params()} network_identity={props.network_identity} />}
        </Show>
      </div>
    </Match>
    <Match when={props.call.decoded.kind === "signature_fallback"}>
      <div class="space-y-3">
        <div class="grid gap-3 md:grid-cols-2">
          <TxField label="Selector" value={props.call.decoded.kind === "signature_fallback" ? props.call.decoded.selector : undefined} />
          <TxField label="Contract" value={props.call.decoded.kind === "signature_fallback" ? props.call.decoded.contract?.verified_name ?? props.call.decoded.contract?.address : undefined} network_identity={props.network_identity} />
        </div>
        <Show when={props.call.decoded.kind === "signature_fallback" ? props.call.decoded.candidates.length > 0 : false}>
          <div class="space-y-2">
            <div class="text-xs font-medium uppercase tracking-wide text-muted">Possible signatures</div>
            <For each={props.call.decoded.kind === "signature_fallback" ? props.call.decoded.candidates : []}>
              {candidate => <TxValue value={candidate} network_identity={props.network_identity} />}
            </For>
          </div>
        </Show>
      </div>
    </Match>
    <Match when={props.call.decoded.kind === "raw"}>
      <TxField label="Raw data" value={props.call.decoded.kind === "raw" ? props.call.decoded.data : undefined} />
    </Match>
  </Switch>
);

export const TxDecodedCall: Component<{ call: DecodedCall; network_identity: number; nested?: boolean; }> = props => (
  <section class={props.nested ? "space-y-2 rounded border border-border/70 bg-background/40 p-2" : "space-y-2"}>
    <div class="grid gap-2 md:grid-cols-[minmax(0,1fr)_minmax(0,1.35fr)] md:items-start">
      <div class="min-w-0 space-y-1">
        <div class="flex min-w-0 flex-wrap items-center gap-2">
          <CallTitle call={props.call} network_identity={props.network_identity} />
          <Show when={props.call.decoded.kind !== "verified"}>
            <code class="rounded bg-foreground/10 px-1.5 py-0.5 text-xs text-muted">{props.call.decoded.kind}</code>
          </Show>
        </div>
        <Show when={props.call.value !== "0"}>
          <TxField label="Value" value={props.call.value} />
        </Show>
      </div>
      <div class="grid min-w-0 grid-cols-[auto_minmax(0,1fr)] items-start gap-2">
        <OperationBadge operation={props.call.operation} />
        <TargetPreview decoded={props.call.decoded} to={props.call.to} network_identity={props.network_identity} />
      </div>
    </div>
    <div class="flex flex-wrap gap-2">
      <Show when={props.call.decoded.kind !== "verified"}>
        <TxField label="Selector" value={props.call.selector} />
      </Show>
    </div>
    <DecodedKind call={props.call} network_identity={props.network_identity} />
    <Show when={subcalls(props.call).length > 0}>
      <div class="space-y-1.5">
        <div class="text-xs font-medium uppercase tracking-wide text-muted">Calls</div>
        <div class="space-y-1.5">
          <For each={subcalls(props.call)}>
            {subcall => <TxDecodedCall call={subcall} network_identity={props.network_identity} nested />}
          </For>
        </div>
      </div>
    </Show>
  </section>
);
