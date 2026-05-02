import { FiArrowLeft, FiPlus } from "solid-icons/fi";
import { Component, createEffect, createMemo, createSignal, Match, Show, Suspense, Switch } from "solid-js";
import { createStore } from "solid-js/store";

import { QuoterConfig, QuoterCreate, useCreateQuoter } from "#/api/quoter";
import { AssetSelect } from "#/components/asset/select";
import { Modal } from "#/components/dialog";
import { button } from "#/components/input/button";

import { QuoterDiscovery } from "./discovery/discovery";

export type QuoterType = "fixed" | "uniswap_v2" | "uniswap_v3" | "erc4626";

const QUOTER_LABELS: Record<QuoterType, string> = {
    fixed: "Fixed",
    uniswap_v2: "Uniswap V2",
    uniswap_v3: "Uniswap V3",
    erc4626: "ERC-4626",
};

const QuoterAddInner: Component<{ onClose: () => void; }> = ({ onClose }) => {
    const [selectedType, setSelectedType] = createSignal<QuoterType | undefined>(undefined);
    const [quoterName, setQuoterName] = createSignal("");
    const [tokenA, setTokenA] = createSignal("");
    const [tokenB, setTokenB] = createSignal("");
    const [enabled, setEnabled] = createSignal(true);
    const [watch, setWatch] = createSignal(true);
    const [fixedPrice, setFixedPrice] = createSignal("1000");
    const [fixedDecimals, setFixedDecimals] = createSignal(3);
    const [tokenInDecimals, setTokenInDecimals] = createSignal(6);
    const [tokenOutDecimals, setTokenOutDecimals] = createSignal(6);

    type FData = { pool?: string; pair?: string; };

    const [data, setData] = createStore<FData>({});

    const tokensSorted = createMemo(() => [tokenA(), tokenB()].sort());

    const tokenBForQuoter = createMemo(() => tokenB());
    const hasPair = createMemo(() => tokenA().length > 0 && tokenB().length > 0);
    const baseFieldsValid = createMemo(() => (
        quoterName().length > 0
        && tokenA().length > 0
        && !!tokenBForQuoter()
    ));
    const canCreate = createMemo(() => {
        const type = selectedType();

        if (!type || !baseFieldsValid()) return false;

        switch (type) {
            case "fixed": {
                return fixedPrice().length > 0 && Number.isFinite(fixedDecimals()) && Number.isFinite(tokenInDecimals()) && Number.isFinite(tokenOutDecimals());
            }
            case "uniswap_v3": {
                return data && data.pool && data.pool.length > 0;
            }
            case "uniswap_v2": {
                return data && data.pair && data.pair.length > 0;
            }
            case "erc4626": {
                return true;
            }
        }
    });

    const config = createMemo((): QuoterConfig | undefined => {
        const type = selectedType();

        switch (type) {
            case "fixed": {
                return {
                    type,
                    price: fixedPrice(),
                    decimals: fixedDecimals(),
                    token_in_decimals: tokenInDecimals(),
                    token_out_decimals: tokenOutDecimals(),
                } as QuoterConfig;
            }
            case "uniswap_v2": {
                return {
                    type,
                    pair_address: data.pair!,
                } as QuoterConfig;
            }
            case "uniswap_v3": {
                return {
                    type,
                    pool_address: data.pool!,
                } as QuoterConfig;
            }
            case "erc4626": {
                return { type } as unknown as QuoterConfig;
            }
            default: {
                return undefined;
            }
        }
    });

    createEffect(() => {
        console.log(config());
    });

    const quoter = createMemo((): QuoterCreate | undefined => {
        const quoterConfig = config();

        if (!canCreate() || !quoterConfig) return undefined;

        return {
            quoter_name: quoterName(),
            token_a: tokenA(),
            token_b: tokenBForQuoter()!,
            config: quoterConfig,
            enabled: enabled(),
            watch: watch(),
        };
    });

    const createQuoter = useCreateQuoter(({ data }: { data: QuoterCreate; }) => ({
        contentType: "application/json; charset=utf-8",
        data,
    }), {
        onSuccess: () => {
            onClose();
        },
    });

    const setTokenASelection = (value: string) => {
        setTokenA(value);
        setSelectedType(undefined);
        setData({});
    };

    const setTokenBSelection = (value: string) => {
        setTokenB(value);
        setSelectedType(undefined);
        setData({});
    };

    const chooseQuoter = (type: QuoterType, options: { pool?: string; pair?: string; token_b?: string; } = {}) => {
        setSelectedType(type);

        if (options.pool) setData({ pool: options.pool });

        if (options.pair) setData({ pair: options.pair });

        if (type === "uniswap_v3") {
            const [token_a, token_b] = tokensSorted();

            if (token_a !== tokenA()) setTokenA(token_a);

            if (token_b !== tokenB()) setTokenB(token_b);
        }

        if (type === "erc4626") {
            setTokenB(options.token_b!);
        }

        if (!quoterName()) setQuoterName(`${QUOTER_LABELS[type]} Quoter`);
    };

    return (
        <>
            <Modal.Title class="flex items-center gap-2">
                <Show when={selectedType()}>
                    <button
                      type="button"
                      aria-label="Back to search"
                      class={button({ variant: "ghost", size: "small", square: true })}
                      onClick={() => {
                            setSelectedType(undefined);
                            setData({});
                        }}
                    >
                        <FiArrowLeft />
                    </button>
                </Show>
                <span>
                    <Show when={selectedType()} fallback="Add Quoter">
                        {type => `Add ${QUOTER_LABELS[type()]} Quoter`}
                    </Show>
                </span>
                <Modal.CloseButton />
            </Modal.Title>
            <div class="px-4 pt-4 space-y-4">
                <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
                    <AssetSelect label="Token A" value={tokenA} onChange={setTokenASelection} />
                    <AssetSelect label="Token B" value={tokenB} onChange={setTokenBSelection} />
                </div>
                <div class="space-y-2">
                    <Show when={selectedType() !== "fixed"}>
                        <Suspense>
                            <QuoterDiscovery
                              token_a={tokenA}
                              token_b={tokenB}
                              data={() => data}
                              onChoose={chooseQuoter}
                              selectedType={() => selectedType()}
                            />
                        </Suspense>
                    </Show>
                    <Show
                      when={selectedType()}
                      fallback={(
                            <div class="space-y-2">
                                <Show when={hasPair()}>
                                    <button
                                      type="button"
                                      class="w-full rounded-md border border-border border-dashed p-4 text-left transition hover:bg-surface-alt"
                                      onClick={() => chooseQuoter("fixed")}
                                    >
                                        <div class="font-medium">Set a price manually</div>
                                        <div class="mt-1 text-sm text-muted">Create a fixed quoter for this pair.</div>
                                    </button>
                                </Show>
                            </div>
                        )}
                    >
                        {type => (
                            <div class="space-y-4">
                                <div class="">
                                    <label class="space-y-1 block w-full">
                                        <span>Name</span>
                                        <input
                                          type="text"
                                          class="input w-full"
                                          value={quoterName()}
                                          onChange={event => setQuoterName(event.target.value)}
                                          placeholder={`${QUOTER_LABELS[type()]} ETH / USDC`}
                                        />
                                    </label>
                                </div>
                                <Switch>
                                    <Match when={type() === "fixed"}>
                                        <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
                                            <label class="space-y-1 block w-full">
                                                <span>Price</span>
                                                <input
                                                  type="text"
                                                  class="input w-full"
                                                  value={fixedPrice()}
                                                  onChange={event => setFixedPrice(event.target.value)}
                                                  placeholder="1000000000000000000"
                                                />
                                            </label>
                                            <label class="space-y-1 block w-full">
                                                <span>Price Decimals</span>
                                                <input
                                                  type="number"
                                                  class="input w-full"
                                                  value={fixedDecimals()}
                                                  onChange={event => setFixedDecimals(Number(event.target.value))}
                                                />
                                            </label>
                                            <label class="space-y-1 block w-full">
                                                <span>Token In Decimals</span>
                                                <input
                                                  type="number"
                                                  class="input w-full"
                                                  value={tokenInDecimals()}
                                                  onChange={event => setTokenInDecimals(Number(event.target.value))}
                                                />
                                            </label>
                                            <label class="space-y-1 block w-full">
                                                <span>Token Out Decimals</span>
                                                <input
                                                  type="number"
                                                  class="input w-full"
                                                  value={tokenOutDecimals()}
                                                  onChange={event => setTokenOutDecimals(Number(event.target.value))}
                                                />
                                            </label>
                                        </div>
                                    </Match>
                                    <Match when={type() === "uniswap_v3"}>
                                        <div class="rounded-md border border-border p-3 text-sm text-muted">
                                            Selected pool:
                                            <span class="ml-1 break-all text-foreground">{data.pool}</span>
                                        </div>
                                    </Match>
                                </Switch>
                                <div class="flex flex-col gap-2 rounded-md border border-border p-3 md:flex-row md:items-center md:justify-between">
                                    <label class="inline-flex items-center gap-2 text-sm">
                                        <input type="checkbox" checked={enabled()} onChange={event => setEnabled(event.target.checked)} />
                                        Enabled
                                    </label>
                                    <label class="inline-flex items-center gap-2 text-sm">
                                        <input type="checkbox" checked={watch()} onChange={event => setWatch(event.target.checked)} />
                                        Watch
                                    </label>
                                </div>
                            </div>
                        )}
                    </Show>
                </div>
            </div>
            <div class="w-full flex justify-end gap-2 p-4">
                <Show when={selectedType()}>
                    <button
                      class={button({ variant: "primary" })}
                      disabled={!quoter() || createQuoter.isPending}
                      onClick={() => createQuoter.mutate({ data: quoter()! })}
                    >
                        Create
                    </button>
                </Show>
            </div>
        </>

    );
};

export const QuoterAdd = () => {
    const [open, setOpen] = createSignal(false);

    return (
        <Modal
          open={open()}
          onOpenChange={setOpen}
        >
            <Modal.Trigger class={button({ variant: "primary" })}>
                Add Quoter
                <FiPlus />
            </Modal.Trigger>
            <Modal.Portal>
                <Modal.Overlay />
                <div class="fixed inset-0 overflow-y-auto p-4">
                    <Modal.Content class="w-full max-w-2xl bg-surface rounded-md relative mx-auto mt-20">
                        <Suspense>
                            <QuoterAddInner onClose={() => setOpen(false)} />
                        </Suspense>
                    </Modal.Content>
                </div>
            </Modal.Portal>
        </Modal>
    );
};
