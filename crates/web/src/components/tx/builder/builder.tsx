import { DropdownMenu } from "@kobalte/core/dropdown-menu";
import { useParams } from "@tanstack/solid-router";
import { FaSolidGripVertical } from "solid-icons/fa";
import { FiPlus, FiTrash2 } from "solid-icons/fi";
import { createMemo, createSignal, For, Match, Show, Suspense, Switch } from "solid-js";

import { useAccount } from "#/api/account";
import { useNetworks } from "#/api/network";
import { button } from "#/components/input/button";
import { NetworkIcon } from "#/components/net/icon";
import { moveItem } from "#/utils/array";
import { createPointerDrag } from "#/utils/pointer-drag";

import { BuilderTx, TX_PRESETS, TX_TYPE_META } from ".";
import { TxApproveBuilder } from "./approve";
import { TxDepositBuilder } from "./deposit";
import { TxRawBuilder } from "./raw";
import { TxSendBuilder } from "./send";
import { TxSwapBuilder } from "./swap";
import { TxWrapBuilder } from "./wrap";

type Props = {
    initialPrefill?: {
        type: BuilderTx["type"];
        data: Record<string, string>;
    };
};

type TxInsertion = {
    index: number;
    edge: "before" | "after";
};

const txInsertionAt = (x: number, y: number): TxInsertion | null => {
    const el = document.elementFromPoint(x, y);
    const row = el?.closest<HTMLElement>("[data-drop-builder-tx]");

    if (!row) return null;

    const index = Number(row.dataset.dropBuilderTx);

    if (!Number.isFinite(index)) return null;

    const { top, height } = row.getBoundingClientRect();

    return {
        index,
        edge: y < top + height / 2 ? "before" : "after",
    };
};

const findTxInsertion = (x: number, y: number, sourceIndex: number) => {
    const insertion = txInsertionAt(x, y);

    return insertion?.index === sourceIndex ? null : insertion;
};

const moveTx = (txs: BuilderTx[], sourceIndex: number, insertion: TxInsertion) => {
    const dropIndex = insertion.index + (insertion.edge === "after" ? 1 : 0);
    const destination = dropIndex > sourceIndex ? dropIndex - 1 : dropIndex;

    return moveItem(txs, sourceIndex, destination);
};

export const TxBuilder = (props: Props) => {
    const params = useParams({ from: "/acc/$account" });
    const accountIdentity = Number.parseInt(params().account);

    const accountQuery = useAccount(() => ({
        path: { account_identity: accountIdentity },
    }));
    const accountNetworks = createMemo(() => accountQuery.data?.networks ?? []);

    const networksQuery = useNetworks();
    const enabledNetworks = createMemo(() => {
        const allNets = networksQuery.data?.networks ?? [];
        const enabledIds = accountNetworks();

        return allNets.filter(n => enabledIds.includes(n.network_identity));
    });

    const [networkIdentity, setNetworkIdentity] = createSignal<number | null>(null);

    const autoSelected = createMemo(() => {
        const networkIdentities = accountNetworks();

        if (networkIdentities.length === 1) {
            const networkId = networkIdentities[0];

            setNetworkIdentity(networkId);

            return true;
        }

        return false;
    });

    const [txData, setTxData] = createSignal<BuilderTx[]>(props.initialPrefill ? [{ type: props.initialPrefill.type, data: props.initialPrefill.data }] : []);
    const [selectedIndex, setSelectedIndex] = createSignal<number | null>(props.initialPrefill ? 0 : null);
    const [insertion, setInsertion] = createSignal<TxInsertion | null>(null);

    const selectedTx = createMemo(() => {
        const idx = selectedIndex();
        const list = txData();

        return idx === null ? undefined : list[idx];
    });

    const supportsMultipleTx = createMemo(() => true);

    const addTx = (type: BuilderTx["type"]) => {
        const newData = setTxData((prev) => {
            const newTx: BuilderTx = { type, data: {} } as BuilderTx;

            return [...prev, newTx];
        });

        setSelectedIndex(newData.length - 1);
    };

    const removeTx = (index: number) => {
        setTxData(prev => prev.filter((_, i) => i !== index));
        const current = selectedIndex();

        if (current === index) {
            setSelectedIndex(current > 0 ? current - 1 : null);
        }
        else if (current !== null && current > index) {
            setSelectedIndex(current - 1);
        }
    };

    const updateTxData = (index: number, data: Record<string, string>) => {
        setTxData(prev => prev.map((tx, i) => (i === index ? { ...tx, data } : tx)));
    };

    const updateTxType = (index: number, newType: BuilderTx["type"]) => {
        setTxData(prev => prev.map((tx, i) => (i === index ? { ...tx, type: newType } : tx)));
    };

    const drag = createPointerDrag<number>({
        onMove: (index, point) => setInsertion(findTxInsertion(point.x, point.y, index)),
        onDrop: (index, point) => {
            const target = insertion() ?? findTxInsertion(point.x, point.y, index);
            const selected = selectedTx();

            if (!target) return;

            const next = moveTx(txData(), index, target);

            setTxData(next);
            setSelectedIndex(selected ? next.indexOf(selected) : null);
        },
        onStop: () => setInsertion(null),
    });

    const handleSubmit = () => {
        const allTx = txData();

        console.log("Submitting transactions:", allTx);
    };

    const hasNetwork = () => networkIdentity() !== null;
    const txLabel = (tx: BuilderTx) => TX_PRESETS.find(p => p.type === tx.type)?.name ?? tx.type;

    return (
        <div class="rounded-md w-full space-y-4 max-w-7xl">
            <Show when={accountQuery.isLoading}>
                <div class="text-muted">Loading account...</div>
            </Show>

            <Show when={!accountQuery.isLoading && !autoSelected() && !hasNetwork()}>
                <Suspense>
                    <Show
                      when={enabledNetworks().length > 0}
                      fallback={(
                            <div class="text-muted p-4 text-center">
                                No networks enabled for this account.
                            </div>
                        )}
                    >
                        <div class="space-y-3">
                            <div class="text-sm text-muted">Select a network</div>
                            <div class="grid grid-cols-2 gap-2">
                                <For each={enabledNetworks()}>
                                    {network => (
                                        <button
                                          class="border bg-surface border-border rounded-md p-4 flex items-center gap-3 hover:bg-surface-alt cursor-pointer"
                                          onClick={() => setNetworkIdentity(network.network_identity)}
                                        >
                                            <NetworkIcon network_identity={network.network_identity} />
                                            <div>
                                                <div class="font-medium">{network.network_name}</div>
                                                <div class="text-xs text-muted">
#
{network.network_identity}
                                                </div>
                                            </div>
                                        </button>
                                    )}
                                </For>
                            </div>
                        </div>
                    </Show>
                </Suspense>
            </Show>

            <Show when={hasNetwork()}>
                <div class="text-sm text-muted flex items-center gap-2">
                    Network:
                    {" "}
                    <span class="flex items-center gap-1">
                        <NetworkIcon network_identity={networkIdentity()!} />
                        {enabledNetworks().find(n => n.network_identity === networkIdentity())?.network_name ?? `#${networkIdentity()}`}
                    </span>
                </div>

                <Show
                  when={selectedIndex() !== null && selectedTx()}
                  fallback={(
                        <div class="grid grid-cols-2 gap-2">
                            <For each={TX_PRESETS}>
                                {item => (
                                    <button
                                      class="border bg-surface border-border rounded-md p-4 flex flex-col justify-center items-start gap-2 hover:bg-surface-alt cursor-pointer"
                                      onClick={() => addTx(item.type)}
                                    >
                                        <span class="flex justify-center items-center gap-2">
                                            <item.icon />
                                            {item.name}
                                        </span>
                                        {item.description && (
                                            <span class="text-muted text-xs">{item.description}</span>
                                        )}
                                    </button>
                                )}
                            </For>
                        </div>
                    )}
                >
                    <div class="flex gap-4 bg-surface p-4 rounded-md">
                        <div class="w-48 shrink-0 space-y-1">
                            <For each={txData()}>
                                {(tx, index) => (
                                    <div
                                      data-drop-builder-tx={index()}
                                      class="relative flex items-center gap-1"
                                      classList={{
                                            "opacity-40 pointer-events-none": drag.dragItem() === index(),
                                        }}
                                    >
                                        <Show when={insertion()?.index === index() && insertion()?.edge === "before"}>
                                            <div class="absolute inset-x-0 top-0 -translate-y-1/2 h-0.5 rounded-full bg-primary pointer-events-none z-10" />
                                        </Show>
                                        <Show when={insertion()?.index === index() && insertion()?.edge === "after"}>
                                            <div class="absolute inset-x-0 bottom-0 translate-y-1/2 h-0.5 rounded-full bg-primary pointer-events-none z-10" />
                                        </Show>
                                        <button
                                          type="button"
                                          class="touch-none shrink-0 rounded p-1 text-muted hover:text-foreground cursor-grab active:cursor-grabbing"
                                          onPointerDown={drag.startDrag(index())}
                                          title="Reorder"
                                        >
                                            <FaSolidGripVertical class="size-3.5" />
                                        </button>
                                        <button
                                          classList={{
                                                "flex-1 text-left border rounded-md px-3 py-2 text-sm transition-colors": true,
                                                "border-primary bg-primary/10": selectedIndex() === index(),
                                                "border-border hover:bg-surface-alt": selectedIndex() !== index(),
                                            }}
                                          onClick={() => setSelectedIndex(index())}
                                        >
                                            {txLabel(tx)}
                                        </button>
                                        <button
                                          class="p-1.5 text-muted hover:text-red-500 hover:bg-red-500/10 rounded-md transition-colors cursor-pointer"
                                          onClick={() => removeTx(index())}
                                          title="Remove"
                                        >
                                            <FiTrash2 class="size-3.5" />
                                        </button>
                                    </div>
                                )}
                            </For>
                            <Show when={supportsMultipleTx()}>
                                <DropdownMenu>
                                    <DropdownMenu.Trigger class="w-full p-2 hover:bg-surface-alt rounded-md flex justify-center items-center border border-dashed border-border text-muted hover:text-foreground transition-colors cursor-pointer">
                                        <FiPlus class="size-4" />
                                    </DropdownMenu.Trigger>
                                    <DropdownMenu.Content class="bg-surface border border-border rounded-md shadow-lg z-50">
                                        <For each={TX_PRESETS}>
                                            {item => (
                                                <DropdownMenu.Item
                                                  class="p-2 hover:bg-surface-alt flex justify-start items-center gap-2 cursor-pointer"
                                                  onClick={() => addTx(item.type)}
                                                >
                                                    <item.icon />
                                                    {item.name}
                                                </DropdownMenu.Item>
                                            )}
                                        </For>
                                    </DropdownMenu.Content>
                                </DropdownMenu>
                            </Show>
                        </div>

                        <div class="flex-1 min-w-0">
                            <Show when={selectedTx()}>
                                {tx => (
                                    <Switch>
                                        <Match when={tx().type === "send"}>
                                            <TxSendBuilder
                                              data={tx().data}
                                              onChange={data => updateTxData(selectedIndex()!, data)}
                                              accountIdentity={accountIdentity}
                                              networkIdentity={networkIdentity()!}
                                            />
                                        </Match>
                                        <Match when={tx().type === "swap"}>
                                            <TxSwapBuilder
                                              data={tx().data}
                                              onChange={data => updateTxData(selectedIndex()!, data)}
                                              accountIdentity={accountIdentity}
                                              networkIdentity={networkIdentity()!}
                                            />
                                        </Match>
                                        <Match when={tx().type === "deposit" || tx().type === "withdraw"}>
                                            <TxDepositBuilder
                                              direction={tx().type as "deposit" | "withdraw"}
                                              data={tx().data}
                                              onChange={data => updateTxData(selectedIndex()!, data)}
                                              onDirectionChange={direction => updateTxType(selectedIndex()!, direction)}
                                              accountIdentity={accountIdentity}
                                              networkIdentity={networkIdentity()!}
                                            />
                                        </Match>
                                        <Match when={tx().type === "approve"}>
                                            <TxApproveBuilder
                                              data={tx().data}
                                              onChange={data => updateTxData(selectedIndex()!, data)}
                                              accountIdentity={accountIdentity}
                                              networkIdentity={networkIdentity()!}
                                            />
                                        </Match>
                                        <Match when={tx().type === "wrap" || tx().type === "unwrap"}>
                                            <TxWrapBuilder
                                              direction={tx().type as "wrap" | "unwrap"}
                                              data={tx().data}
                                              onChange={data => updateTxData(selectedIndex()!, data)}
                                              onDirectionChange={direction => updateTxType(selectedIndex()!, direction)}
                                              accountIdentity={accountIdentity}
                                              networkIdentity={networkIdentity()!}
                                            />
                                        </Match>
                                        <Match when={tx().type === "raw"}>
                                            <TxRawBuilder
                                              data={tx().data}
                                              onChange={data => updateTxData(selectedIndex()!, data)}
                                              accountIdentity={accountIdentity}
                                              networkIdentity={networkIdentity()!}
                                            />
                                        </Match>
                                    </Switch>
                                )}
                            </Show>
                        </div>
                    </div>

                    <Show when={drag.dragItem() !== null && drag.pointer()}>
                        <div
                          class="fixed z-50 pointer-events-none rounded-md border border-primary/30 bg-surface px-3 py-2 text-sm shadow-lg"
                          style={{
                                left: `${Math.min(drag.pointer()!.x + 12, globalThis.innerWidth - 220)}px`,
                                top: `${drag.pointer()!.y + 12}px`,
                            }}
                        >
                            {txLabel(txData()[drag.dragItem()!])}
                        </div>
                    </Show>

                    <Show when={txData().length > 0}>
                        <div class="flex justify-end">
                            <button
                              type="button"
                              class={button({ variant: "primary" })}
                              onClick={handleSubmit}
                            >
                                Submit
                                {" "}
                                {txData().length > 1 ? `(${txData().length} transactions)` : ""}
                            </button>
                        </div>
                    </Show>
                </Show>
            </Show>
        </div>
    );
};
