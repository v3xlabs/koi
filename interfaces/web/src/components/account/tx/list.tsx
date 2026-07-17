import { Collapsible } from "@kobalte/core/collapsible";
import { Tooltip } from "@kobalte/core/tooltip";
import { FaSolidRefresh } from "solid-icons/fa";
import { FiCheckSquare, FiChevronDown, FiDollarSign, FiHash, FiLayers, FiZap } from "solid-icons/fi";
import { Component, createMemo, For, JSX, Show, Suspense } from "solid-js";

import { AccountTx, useAccountTxAll } from "#/api/account/tx";
import { button } from "#/components/input/button";
import { AddressPreview } from "#/components/link/address";
import { TxHashPreview } from "#/components/link/txhash";
import { NetworkIcon } from "#/components/net/icon";
import { FormattedTime } from "#/components/time";
import { truncateAddress } from "#/utils/address";

import { TxDecodedCall } from "./decoded-call";
import { TxOrigin } from "./origin";
import { isRecord, titleFromKey, TxField, TxJsonValue } from "./value";

const getSafeExtraValue = (tx: AccountTx, key: string) => {
  const extra = tx.extra.safe_wallet?.extra;

  return isRecord(extra) ? extra[key] : undefined;
};

const getSafeExtraArray = (tx: AccountTx, key: string) => {
  const value = getSafeExtraValue(tx, key);

  return Array.isArray(value) ? value : [];
};

const getRecordString = (record: Record<string, unknown>, key: string) => {
  const value = record[key];

  return typeof value === "string" ? value : undefined;
};

const effectiveSafeDate = (tx: AccountTx) => tx.extra.safe_wallet?.execution_date ?? getSafeExtraValue(tx, "executionDate");
const effectiveSafeExecuted = (tx: AccountTx) => tx.extra.safe_wallet?.is_executed ?? getSafeExtraValue(tx, "isExecuted");
const effectiveSafeSuccessful = (tx: AccountTx) => tx.extra.safe_wallet?.is_successful ?? getSafeExtraValue(tx, "isSuccessful");

const decodedSummary = (tx: AccountTx) => {
  const decoded = tx.decoded?.decoded;

  if (!decoded) return "Unknown action";

  if (decoded.kind === "verified") return `${decoded.function} on ${decoded.contract.verified_name ?? truncateAddress(decoded.contract.address)}`;

  if (decoded.kind === "signature_fallback") return `Unknown call ${decoded.selector}`;

  return "Raw call";
};

const decodedMethod = (tx: AccountTx) => {
  const decoded = tx.decoded?.decoded;

  if (!decoded) return "Unknown";

  if (decoded.kind === "verified") return decoded.function;

  if (decoded.kind === "signature_fallback") return decoded.selector;

  return "Raw";
};

const decodedContract = (tx: AccountTx) => {
  const decoded = tx.decoded?.decoded;

  if (!decoded || decoded.kind === "raw") return truncateAddress(tx.to);

  return decoded.contract?.verified_name ?? truncateAddress(decoded.contract?.address);
};

const subcallCount = (tx: AccountTx) => tx.decoded?.subcalls.length ?? 0;

const summaryTitle = (tx: AccountTx) => {
  const count = subcallCount(tx);

  if (count > 0) return `${count} calls via ${decodedMethod(tx)}`;

  return decodedContract(tx) || decodedSummary(tx);
};

const summarySubtitle = (tx: AccountTx) => {
  const count = subcallCount(tx);

  if (count > 0) return decodedContract(tx);

  return tx.tx_hash ? truncateAddress(tx.tx_hash) : decodedSummary(tx);
};

const statusClass = (tx: AccountTx) => {
  const successful = effectiveSafeSuccessful(tx);
  const executed = effectiveSafeExecuted(tx);

  if (successful === true) return "text-emerald-400";

  if (successful === false) return "text-red-400";

  if (executed === true) return "text-amber-300";

  return "text-muted";
};

const statusLabel = (tx: AccountTx) => {
  const successful = effectiveSafeSuccessful(tx);
  const executed = effectiveSafeExecuted(tx);

  if (successful === true) return "";

  if (successful === false) return "Failed";

  if (executed === true) return "Executed";

  if (executed === false) return "Not executed";

  return "Unknown";
};

const CompactHashField: Component<{ label: string; value: string | undefined; }> = props => (
  <Show when={props.value}>
    {value => (
      <div class="min-w-0 space-y-1">
        <div class="text-[11px] font-medium uppercase tracking-wide text-muted">{props.label}</div>
        <code class="block truncate rounded bg-foreground/10 px-1.5 py-0.5 text-xs" title={value()}>
          {value()}
        </code>
      </div>
    )}
  </Show>
);

const CompactTxHashField: Component<{ label: string; value: string | undefined; network_identity: number; }> = props => (
  <Show when={props.value}>
    {value => (
      <div class="min-w-0 space-y-1">
        <div class="text-[11px] font-medium uppercase tracking-wide text-muted">{props.label}</div>
        <TxHashPreview txhash={value()} network_identity={props.network_identity} truncate={false} />
      </div>
    )}
  </Show>
);

const CompactNetworkField: Component<{ network_identity: number; }> = props => (
  <div class="min-w-0 space-y-1">
    <div class="text-[11px] font-medium uppercase tracking-wide text-muted">Network</div>
    <div class="inline-flex max-w-full items-center gap-1.5 rounded bg-foreground/10 px-1.5 py-0.5 text-xs">
      <NetworkIcon network_identity={props.network_identity} />
      <code class="truncate bg-transparent p-0">{props.network_identity}</code>
    </div>
  </div>
);

const TimelineMetric: Component<{ label: string; value: unknown; icon: Component<{ class?: string; }>; }> = props => (
  <Show when={props.value !== undefined && props.value !== null}>
    <Tooltip placement="top">
      <Tooltip.Trigger class="inline-flex items-center gap-1 rounded bg-foreground/10 px-1.5 py-0.5 text-xs text-muted">
        <props.icon class="size-3" />
        <span>{String(props.value)}</span>
      </Tooltip.Trigger>
      <Tooltip.Portal>
        <Tooltip.Content class="bg-surface-alt text-secondary-foreground rounded-md border border-border p-2 text-xs">
          <Tooltip.Arrow />
          {props.label}
        </Tooltip.Content>
      </Tooltip.Portal>
    </Tooltip>
  </Show>
);

const TimelineEntry: Component<{ label: string; address?: string; time?: unknown; detail?: string; network_identity: number; children?: JSX.Element; }> = props => (
  <div class="grid grid-cols-[1rem_minmax(0,1fr)] gap-3">
    <div class="relative flex justify-center">
      <div class="mt-1 size-2 rounded-full bg-primary" />
      <div class="absolute top-4 bottom-[-1rem] w-px bg-border last:hidden" />
    </div>
    <div class="space-y-1 pb-4">
      <div class="flex flex-wrap items-center justify-between gap-2">
        <div class="text-sm font-medium">{props.label}</div>
        <FormattedTime value={props.time as string | number | Date | null | undefined} class="text-xs text-muted" />
      </div>
      <Show when={props.address}>
        {address => <AddressPreview address={address()} network_identity={props.network_identity} />}
      </Show>
      <Show when={props.detail}>
        {detail => <div class="text-xs text-muted">{detail()}</div>}
      </Show>
      <Show when={props.children}>
        <div class="flex flex-wrap gap-1.5 pt-1">
          {props.children}
        </div>
      </Show>
    </div>
  </div>
);

const SafeTimeline: Component<{ tx: AccountTx; }> = (props) => {
  const safe = createMemo(() => props.tx.extra.safe_wallet);
  const confirmations = createMemo(() => getSafeExtraArray(props.tx, "confirmations").filter(isRecord));
  const submissionDate = createMemo(() => getSafeExtraValue(props.tx, "submissionDate"));
  const confirmationsRequired = createMemo(() => getSafeExtraValue(props.tx, "confirmationsRequired"));

  return (
    <Show when={safe()}>
      {safeWallet => (
        <aside class="space-y-3 rounded border border-border/70 bg-background/40 p-3">
          <div class="flex items-center justify-between gap-3">
            <div class="text-sm font-medium">Timeline</div>
            <Show when={confirmationsRequired()}>
              {required => (
                <div class="text-xs text-muted">
                  {confirmations().length}
                  /
                  {String(required())}
                  {" "}
                  signed
                </div>
              )}
            </Show>
          </div>
          <div>
            <TimelineEntry
              label="Proposed"
              address={safeWallet().proposer}
              time={submissionDate()}
              network_identity={props.tx.network_identity}
            />
            <For each={confirmations()}>
              {confirmation => (
                <TimelineEntry
                  label="Signed"
                  address={getRecordString(confirmation, "owner")}
                  time={confirmation.submissionDate}
                  detail={getRecordString(confirmation, "signatureType")}
                  network_identity={props.tx.network_identity}
                />
              )}
            </For>
            <Show when={effectiveSafeExecuted(props.tx) || safeWallet().executor}>
              <TimelineEntry
                label="Executed"
                address={safeWallet().executor}
                time={effectiveSafeDate(props.tx)}
                network_identity={props.tx.network_identity}
              >
                <TimelineMetric label="Block number" value={getSafeExtraValue(props.tx, "blockNumber")} icon={FiHash} />
                <TimelineMetric label="Gas used" value={getSafeExtraValue(props.tx, "gasUsed")} icon={FiZap} />
                <TimelineMetric label="Fee" value={getSafeExtraValue(props.tx, "fee")} icon={FiDollarSign} />
                <TimelineMetric label="Confirmations required" value={confirmationsRequired()} icon={FiCheckSquare} />
              </TimelineEntry>
            </Show>
          </div>
        </aside>
      )}
    </Show>
  );
};

const TransactionMeta: Component<{ tx: AccountTx; }> = props => (
  <aside>
    <div class="text-sm font-medium">Transaction</div>
    <div class="space-y-3">
      <CompactTxHashField label="Tx hash" value={props.tx.tx_hash} network_identity={props.tx.network_identity} />
      <CompactHashField label="Safe tx hash" value={props.tx.extra.safe_wallet?.safe_tx_hash} />
      <CompactNetworkField network_identity={props.tx.network_identity} />
    </div>
  </aside>
);

const TransactionSidePanel: Component<{ tx: AccountTx; }> = props => (
  <div class="space-y-3">
    <TransactionMeta tx={props.tx} />
    <SafeTimeline tx={props.tx} />
    <Show when={props.tx.extra.safe_wallet?.origin}>
      {origin => (
        <TxOrigin origin={origin()} />
      )}
    </Show>
  </div>
);

const TransactionDetails: Component<{ tx: AccountTx; }> = props => (
  <div class="divide-y divide-border/70">
    <div class="grid gap-4 p-4 lg:grid-cols-[minmax(0,1fr)_16rem]">
      <div class="min-w-0 space-y-5">
        <section class="space-y-3">
          <Show when={!props.tx.decoded}>
            <div class="grid gap-3 md:grid-cols-2">
              <TxField label="From" value={props.tx.from} network_identity={props.tx.network_identity} />
              <TxField label="To" value={props.tx.to} network_identity={props.tx.network_identity} />
              <TxField label="Value" value={props.tx.value} />
              <TxField label="Data" value={props.tx.data} />
            </div>
          </Show>
        </section>
        <Show when={props.tx.decoded}>
          {decoded => (
            <section class="space-y-3">
              <TxDecodedCall call={decoded()} network_identity={props.tx.network_identity} />
            </section>
          )}
        </Show>
      </div>
      <TransactionSidePanel tx={props.tx} />
    </div>
    <Collapsible>
      <Collapsible.Trigger class="w-full px-4 py-3 text-left text-sm text-primary hover:bg-foreground/5">
        Full API response
      </Collapsible.Trigger>
      <Collapsible.Content class="px-4 pb-4">
        <div class="space-y-2">
          <For each={Object.entries(props.tx)}>
            {entry => (
              <div class="grid gap-2 rounded border border-border/70 bg-background/40 p-3 md:grid-cols-[12rem_minmax(0,1fr)]">
                <div class="text-xs font-medium uppercase tracking-wide text-muted">{titleFromKey(entry[0])}</div>
                <TxJsonValue value={entry[1]} network_identity={props.tx.network_identity} />
              </div>
            )}
          </For>
        </div>
      </Collapsible.Content>
    </Collapsible>
  </div>
);

const TransactionCard: Component<{ tx: AccountTx; }> = props => (
  <div class="overflow-hidden">
    <Collapsible>
      <Collapsible.Trigger class="w-full cursor-pointer p-3 text-left hover:outline-2 hover:-outline-offset-2 hover:outline-border">
        <div class="grid w-full grid-cols-[auto_minmax(0,1fr)_auto_auto] items-center gap-3 md:grid-cols-[14rem_minmax(0,1fr)_auto_auto_auto]">
          <div class="flex items-center gap-3">
            <div class="min-w-8 text-sm font-medium text-muted">
              #
              {props.tx.extra.safe_wallet?.nonce ?? "-"}
            </div>
            <Show
              when={subcallCount(props.tx) > 0}
              fallback={(
                <div class="min-w-0 rounded bg-background/70 px-2 py-1 text-xs text-muted">
                  <span class="block truncate">{decodedMethod(props.tx)}</span>
                </div>
              )}
            >
              <div class="flex items-center gap-1 rounded bg-background/70 px-2 py-1 text-xs text-muted">
                <FiLayers />
                Multicall
              </div>
            </Show>
          </div>
          <div class="min-w-0">
            <div class="truncate font-medium">{summaryTitle(props.tx)}</div>
          </div>
          <div>
            <Show
              when={props.tx.tx_hash}
              fallback={<div class="truncate text-xs text-muted">{summarySubtitle(props.tx)}</div>}
            >
              {txhash => (
                <div class="mt-0.5" onClick={event => event.stopPropagation()}>
                  <TxHashPreview txhash={txhash()} network_identity={props.tx.network_identity} />
                </div>
              )}
            </Show>
          </div>
          <div class="hidden text-sm text-muted md:block">
            <FormattedTime value={effectiveSafeDate(props.tx) as string | number | Date | null | undefined} />
          </div>
          <div class="flex items-center gap-2 text-sm">
            <span class={statusClass(props.tx)}>{statusLabel(props.tx)}</span>
            <FiChevronDown />
          </div>
        </div>
      </Collapsible.Trigger>
      <Collapsible.Content>
        <TransactionDetails tx={props.tx} />
      </Collapsible.Content>
    </Collapsible>
  </div>
);

export const AccountTxHistory: Component<{ account_identity: number; }> = (props) => {
  const allQuery = useAccountTxAll(() => ({ path: { account_identity: props.account_identity } }));

  return (
    <div class="flex h-full w-full flex-col bg-surface rounded-lg">
      <div class="flex justify-between p-3 border-b border-border items-center text-sm">
        <div>
          Transactions
        </div>
        <div>
          <button
            class={button({ variant: "ghost", size: "small", square: true })}
            onClick={() => { void allQuery.refetch(); }}
          >
            <FaSolidRefresh />
          </button>
        </div>
      </div>
      <Suspense fallback={<div>Loading...</div>}>
        <Show when={allQuery.data}>
          {data => (
            <Show when={data().transactions.length > 0} fallback={<div class="text-sm text-muted p-4">No transactions found.</div>}>
              <div class="h-full w-full divide-y divide-border overflow-y-auto wrap-anywhere">
                <For each={data().transactions}>
                  {transaction => <TransactionCard tx={transaction} />}
                </For>
              </div>
            </Show>
          )}
        </Show>
      </Suspense>
    </div>
  );
};
