import { SegmentedControl } from "@kobalte/core/segmented-control";
import { createEffect, createMemo, createSignal, For, Show } from "solid-js";
import { match } from "ts-pattern";

import { Asset, useAssetMetadataDiscovery, useCreateAsset } from "#/api/asset";

import { Modal } from "../dialog";
import { AddressInput } from "../input/address";
import { NetworkSelect } from "../net/input";

const PLACEHOLDERS: Record<string, {
    name: string;
    symbol: string;
    decimals: number;
    iconUrl: string;
}> = {
    erc20: {
        name: "Wrapped Ether",
        symbol: "wETH",
        decimals: 18,
        iconUrl: "https://...",
    },
    native: {
        name: "Ether",
        symbol: "ETH",
        decimals: 18,
        iconUrl: "https://...",
    },
    fiat: {
        name: "US Dollar",
        symbol: "USD",
        decimals: 6,
        iconUrl: "https://...",
    },
};

export const AssetAdd = () => {
    const [assetType, setAssetType] = createSignal<string>("erc20");
    const [assetAddress, setAssetAddress] = createSignal("");
    const [networkId, setNetworkId] = createSignal(0);

    const [assetName, setAssetName] = createSignal("");
    const [assetSymbol, setAssetSymbol] = createSignal("");
    const [assetDecimals, setAssetDecimals] = createSignal<number | undefined>(undefined);
    const [assetIconUrl, setAssetIconUrl] = createSignal("");

    const asset = createMemo((): Asset | undefined => {
        const asset_identity = match(assetType())
            .with("erc20", () => (networkId() && assetAddress() ? `erc20:${networkId()}:${assetAddress()}` : undefined))
            .with("native", () => (networkId() ? `native:${networkId()}` : undefined))
            .with("fiat", () => (assetSymbol() ? `fiat:${assetSymbol()}` : undefined))
            .otherwise(() => undefined);

        if (!asset_identity) return undefined;

        const asset_decimals = assetDecimals();

        if (asset_decimals === undefined) return undefined;

        return {
            asset_identity,
            asset_name: assetName(),
            asset_symbol: assetSymbol(),
            asset_decimals,
            asset_icon_url: assetIconUrl(),
        };
    });
    const assetCreate = useCreateAsset(({ data }: { data: Asset; }) => ({ contentType: "application/json; charset=utf-8", data }));

    createEffect(() => {
        console.log(asset());
    });

    const discoveryQuery = useAssetMetadataDiscovery(() => ({ path: { asset_identity: `erc20:${networkId()}:${assetAddress()}` } }), {
        enabled: () => !!assetAddress() && assetType() === "erc20" && networkId() !== 0,
    });

    createEffect(() => {
        console.log(JSON.stringify(discoveryQuery.data?.options));
    });

    const nameSuggestions = createMemo(() => Object.entries(discoveryQuery.data?.options ?? {}).map(
        ([source, value]) => {
            if (value.name) {
                return [value.name, source];
            }

            return null;
        })
        .filter(s => !!s),
    );
    const symbolSuggestions = createMemo(() => Object.entries(discoveryQuery.data?.options ?? {}).map(
        ([source, value]) => {
            if (value.symbol) {
                return [value.symbol, source];
            }

            return null;
        })
        .filter(s => !!s),
    );
    const decimalsSuggestions = createMemo(() => Object.entries(discoveryQuery.data?.options ?? {}).map(
        ([source, value]) => {
            if (value.decimals) {
                return [value.decimals, source];
            }

            return null;
        })
        .filter(s => !!s),
    );
    const iconSuggestions = createMemo(() => Object.entries(discoveryQuery.data?.options ?? {}).map(
        ([source, value]) => {
            console.log(source, value);

            if (value.icon_url) {
                return [value.icon_url, source];
            }

            return null;
        })
        .filter(s => !!s),
    );

    return (
        <Modal>
            <Modal.Trigger class="btn btn-primary">
                Add Asset
            </Modal.Trigger>
            <Modal.Portal>
                <Modal.Overlay />
                <div class="fixed inset-0">
                    <Modal.Content class="w-full max-w-xl bg-surface rounded-md relative mx-auto mt-24">
                        <Modal.Title>
                            Add Asset
                        </Modal.Title>
                        <div class="px-4 pt-4">
                            <div class="w-full flex flex-col gap-2 md:flex-row">
                                <SegmentedControl
                                  value={assetType()}
                                  onChange={setAssetType}
                                  class=""
                                >
                                    <SegmentedControl.Label class="w-full">
                                        Type
                                    </SegmentedControl.Label>
                                    <div class="relative border border-border rounded-md p-1 w-fit" role="presentation">
                                        <SegmentedControl.Indicator class="absolute top-1 left-1 w-full h-full bg-primary rounded-md transition-all duration-300" />
                                        <div class="flex gap-2 w-fit relative">
                                            <For each={Object.keys(PLACEHOLDERS)}>
                                                {key => (
                                                    <SegmentedControl.Item
                                                      value={key}
                                                      class="px-2"
                                                    >
                                                        <SegmentedControl.ItemInput class="" />
                                                        <SegmentedControl.ItemLabel class="cursor-pointer">
                                                            {key}
                                                        </SegmentedControl.ItemLabel>
                                                    </SegmentedControl.Item>
                                                )}
                                            </For>
                                        </div>
                                    </div>
                                </SegmentedControl>
                                <Show when={assetType() === "erc20" || assetType() === "native"}>
                                    <label class="space-y-1 block w-full">
                                        <span>Network</span>
                                        {/* TODO: make single network select instead of this hack lmao */}
                                        <NetworkSelect value={() => [networkId()]} onChange={x => (x ? x[0] && setNetworkId(x.at(-1)!) : setNetworkId(0))} />
                                    </label>
                                </Show>
                            </div>
                            <Show when={assetType() === "erc20"}>
                                <label class="space-y-1 block w-full">
                                    <span>Address</span>
                                    <AddressInput
                                      type="text"
                                      class="input w-full"
                                      value={assetAddress}
                                      onChange={e => setAssetAddress(e)}
                                      placeholder="0x..."
                                    />
                                </label>
                            </Show>
                            <label class="space-y-1 block w-full">
                                <span>Name</span>
                                <input
                                  type="text"
                                  class="input w-full"
                                  value={assetName()}
                                  onChange={e => setAssetName(e.target.value)}
                                  placeholder={PLACEHOLDERS[assetType()].name}
                                />
                                <div>
                                    <For each={nameSuggestions()}>
                                        {([name, source]) => (
                                            <li>
                                                <button onClick={() => setAssetName(name)}>{name}</button>
                                            </li>
                                        )}
                                    </For>
                                </div>
                            </label>
                            <div class="w-full flex flex-col gap-2 md:flex-row">
                                <label class="space-y-1 block w-full">
                                    <span>Symbol</span>
                                    <input
                                      type="text"
                                      class="input w-full"
                                      value={assetSymbol()}
                                      onChange={e => setAssetSymbol(e.target.value)}
                                      placeholder={PLACEHOLDERS[assetType()].symbol}
                                    />
                                    <div>
                                        <For each={symbolSuggestions()}>
                                            {([symbol, source]) => (
                                                <li>
                                                    <button onClick={() => setAssetSymbol(symbol)}>{symbol}</button>
                                                </li>
                                            )}
                                        </For>
                                    </div>
                                </label>
                                <label class="space-y-1 block w-full">
                                    <span>Decimals</span>
                                    <input
                                      type="number"
                                      class="input w-full"
                                      value={assetDecimals()}
                                      onChange={e => setAssetDecimals(Number(e.target.value))}
                                      placeholder={PLACEHOLDERS[assetType()].decimals.toString()}
                                    />
                                    <div>
                                        <For each={decimalsSuggestions()}>
                                            {([decimals, source]) => (
                                                <li>
                                                    <button onClick={() => setAssetDecimals(decimals)}>{decimals}</button>
                                                </li>
                                            )}
                                        </For>
                                    </div>
                                </label>
                            </div>
                            <label class="space-y-1 block w-full">
                                <span>Icon URL</span>
                                <input
                                  type="text"
                                  class="input w-full"
                                  value={assetIconUrl()}
                                  onChange={e => setAssetIconUrl(e.target.value)}
                                  placeholder={PLACEHOLDERS[assetType()].iconUrl}
                                />
                                <ul>
                                    <For each={iconSuggestions()}>
                                        {([iconUrl, source]) => (
                                            <li>
                                                <button onClick={() => setAssetIconUrl(iconUrl)}>
                                                    <img src={iconUrl} alt={source} class="size-8 aspect-square rounded-full" />
                                                    {source}
                                                </button>
                                            </li>
                                        )}
                                    </For>
                                </ul>
                            </label>
                        </div>
                        <div>
                            <Show when={asset()}>
                                {asset => (
                                    <div>
                                        <div class="text-sm wrap-anywhere">
                                            {JSON.stringify(asset())}
                                        </div>
                                    </div>
                                )}
                            </Show>
                        </div>
                        <div class="w-full flex justify-end gap-2 p-4">
                            <button
                              class="btn btn-primary"
                              onClick={() => assetCreate.mutate({ data: asset()! })}
                              disabled={!asset()}
                            >
                                Create
                            </button>
                            <Modal.CloseButton class="btn btn-secondary">
                                Cancel
                            </Modal.CloseButton>
                        </div>
                    </Modal.Content>
                </div>
            </Modal.Portal>
        </Modal>
    );
};
