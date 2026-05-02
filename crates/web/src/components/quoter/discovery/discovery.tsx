import { Accessor, Component, createEffect, createMemo, For, Show, Suspense } from "solid-js";

import { useAsset } from "#/api/asset";
import { UniswapV2Pair, UniswapV3Pool, useDiscoverQuoter } from "#/api/quoter";
import uniswapV3Logo from "#/assets/uniswap.svg";
import { AssetAmount } from "#/components/asset/amount";
import { AssetPreview } from "#/components/asset/preview";

import { QuoterType } from "../add";

export type QuoterDiscoveryProps = {
    token_a: Accessor<string>;
    token_b: Accessor<string>;
    onChoose: (type: "erc4626" | "uniswap_v2" | "uniswap_v3", data: { token_b?: string; pool?: string; pair?: string; }) => void;
    data: Accessor<{ pair?: string; pool?: string; }>;
    selectedType: Accessor<QuoterType | undefined>;
};

const PoolReservePreview = (props: { pool: UniswapV3Pool; token_a: string; token_b: string; }) => {
    const tokenAQuery = useAsset(() => ({ path: { asset_identity: props.token_a } }));
    const tokenBQuery = useAsset(() => ({ path: { asset_identity: props.token_b } }));

    return (
        <div class="mt-2 grid grid-cols-1 gap-2 text-sm text-muted md:grid-cols-2">
            <div>
                <span class="text-foreground">Reserve A:</span>
                {" "}
                <AssetAmount amount={() => BigInt(props.pool.reserve_0 ?? "0")} asset={() => tokenAQuery.data?.asset_identity ?? ""} />
            </div>
            <div>
                <span class="text-foreground">Reserve B:</span>
                <AssetAmount amount={() => BigInt(props.pool.reserve_1 ?? "0")} asset={() => tokenBQuery.data?.asset_identity ?? ""} />
            </div>
        </div>
    );
};

const PairReservePreview = (props: { pair: UniswapV2Pair; token_a: string; token_b: string; }) => {
    const tokenAQuery = useAsset(() => ({ path: { asset_identity: props.token_a } }));
    const tokenBQuery = useAsset(() => ({ path: { asset_identity: props.token_b } }));

    return (
        <div class="mt-2 grid grid-cols-1 gap-2 text-sm text-muted md:grid-cols-2">
            <div>
                <span class="text-foreground">Reserve A:</span>
                {" "}
                <AssetAmount amount={() => BigInt(props.pair.reserve_0 ?? "0")} asset={() => tokenAQuery.data?.asset_identity ?? ""} />
            </div>
            <div>
                <span class="text-foreground">Reserve B:</span>
                {" "}
                <AssetAmount amount={() => BigInt(props.pair.reserve_1 ?? "0")} asset={() => tokenBQuery.data?.asset_identity ?? ""} />
            </div>
        </div>
    );
};

export const QuoterDiscovery: Component<QuoterDiscoveryProps> = ({ token_a, token_b, onChoose, data: _data, selectedType }) => {
    const tokensSorted = createMemo(() => [token_a(), token_b()].sort());
    const discoveryQuery = useDiscoverQuoter(() => ({
        contentType: "application/json; charset=utf-8",
        data: {
            token_a: token_a(),
            token_b: token_b() || undefined,
        },
    }), {
        enabled: () => token_a().length > 0,
    });

    createEffect(() => {
        console.log(discoveryQuery.data);
    });

    return (
        <Show when={discoveryQuery.data}>
            {data => (
                <>
                    <Show when={selectedType() === "erc4626" || selectedType() === undefined}>
                        <Show when={data().erc4626}>
                            {erc4626 => (
                                <button
                                  type="button"
                                  class="w-full rounded-md border border-border bg-surface-alt/40 p-4 text-left transition hover:bg-surface-alt"
                                  onClick={() => onChoose("erc4626", { token_b: erc4626() })}
                                >
                                    <div class="font-medium">ERC-4626 vault detected</div>
                                    <div class="mt-2 flex flex-wrap items-center gap-2 text-sm text-muted">
                                        <AssetPreview asset_identity={token_a()} />
                                        <span>underlying</span>
                                        <AssetPreview asset_identity={erc4626()} />
                                    </div>
                                </button>
                            )}
                        </Show>
                    </Show>
                    <Show when={selectedType() === "uniswap_v2" || selectedType() === undefined}>
                        <Show when={data().uniswap_v2}>
                            {uniswap_v2 => (
                                <button
                                  type="button"
                                  class="w-full rounded-md border border-border bg-surface-alt/40 p-4 text-left transition hover:bg-surface-alt"
                                  onClick={() => onChoose("uniswap_v2", { pair: uniswap_v2().pair_address })}
                                >
                                    <div class="flex items-center justify-between gap-3">
                                        <div class="flex items-center gap-2">
                                            <img src={uniswapV3Logo} alt="Uniswap V2" class="w-4 h-4" />
                                            <div class="font-medium text-sm">Uniswap V2 pair</div>
                                        </div>
                                        <div class="text-sm text-muted">

                                        </div>
                                    </div>
                                    <div class="mt-1 break-all text-sm text-muted">{uniswap_v2().pair_address}</div>
                                    <Show when={tokensSorted()}>
                                        {tokens => (
                                            <Suspense>
                                                <PairReservePreview pair={uniswap_v2()} token_a={tokens()[0]} token_b={tokens()[1]} />
                                            </Suspense>
                                        )}
                                    </Show>
                                </button>
                            )}
                        </Show>
                    </Show>
                    <Show when={selectedType() === "uniswap_v3" || selectedType() === undefined}>
                        <Show when={data().uniswap_v3}>
                            {uniswap_v3 => (
                                <For each={uniswap_v3().filter(_pool => (_data().pool ? _pool.pool_address === _data().pool : true))}>
                                    {pool => (
                                        <button
                                          type="button"
                                          class="w-full rounded-md border border-border bg-surface-alt/40 p-4 text-left transition hover:bg-surface-alt"
                                          onClick={() => onChoose("uniswap_v3", { pool: pool.pool_address })}
                                        >
                                            <div class="flex items-center justify-between gap-3">
                                                <div class="flex items-center gap-2">
                                                    <img src={uniswapV3Logo} alt="Uniswap V3" class="w-4 h-4" />
                                                    <div class="font-medium text-sm">Uniswap V3 pool</div>
                                                </div>
                                                <div class="text-sm text-muted">
                                                    {pool.fee / 10_000}
                                                    %
                                                </div>
                                            </div>
                                            <div class="mt-1 break-all text-sm text-muted">{pool.pool_address}</div>
                                            <Show when={tokensSorted()}>
                                                {tokens => (
                                                    <Suspense>
                                                        <PoolReservePreview pool={pool} token_a={tokens()[0]} token_b={tokens()[1]} />
                                                    </Suspense>
                                                )}
                                            </Show>
                                        </button>
                                    )}
                                </For>
                            )}
                        </Show>
                    </Show>
                </>
            )}
        </Show>
    );
};
