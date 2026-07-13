import { FiPlus } from "solid-icons/fi";
import { Component, createMemo, createSignal, For, JSX, Show } from "solid-js";
import { match } from "ts-pattern";

import { Asset, useAssetMetadataDiscovery, useCreateAsset } from "#/api/asset";
import { CachedImage } from "#/utils/image-cache";

import { Modal } from "../dialog";
import { AddressInput } from "../input/address";
import { button } from "../input/button";
import { SegmentedControl } from "../input/segmented";
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

type AssetAddInnerProps = {
    onClose: () => void;
    onSuccess?: (asset: Asset) => void;
};

const AssetAddInner: Component<AssetAddInnerProps> = (props) => {
    const [assetType, setAssetType] = createSignal<string>("erc20");
    const [assetAddress, setAssetAddress] = createSignal("");
    const [networkId, setNetworkId] = createSignal(0);

    const [assetName, setAssetName] = createSignal("");
    const [assetSymbol, setAssetSymbol] = createSignal("");
    const [assetDecimals, setAssetDecimals] = createSignal<number | undefined>(undefined);
    const [assetIconUrl, setAssetIconUrl] = createSignal("");

    const assetIdentity = createMemo(() => match(assetType())
        .with("erc20", () => (networkId() && assetAddress() ? `erc20:${networkId()}:${assetAddress()}` : undefined))
        .with("native", () => (networkId() ? `native:${networkId()}` : undefined))
        .with("fiat", () => (assetSymbol() ? `fiat:${assetSymbol()}` : undefined))
        .otherwise(() => undefined));
    const asset = createMemo((): Asset | undefined => {
        const asset_identity = assetIdentity();

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
    const assetCreate = useCreateAsset(({ data }: { data: Asset; }) => ({ contentType: "application/json; charset=utf-8", data }), {
        onSuccess: (created) => {
            props.onSuccess?.(created);
            props.onClose();
        },
    });

    const discoveryQuery = useAssetMetadataDiscovery(() => ({ path: { asset_identity: assetIdentity()! } }), {
        enabled: () => !!assetIdentity(),
    });

    const nameSuggestions = createMemo(() => (assetName().length > 0
        ? []
        : Object.entries(discoveryQuery.data?.options ?? {}).map(
            ([source, value]) => {
                if (value.name) {
                    return [value.name, source];
                }

                return null;
            })
            .filter(s => !!s)),
    );
    const symbolSuggestions = createMemo(() => (assetSymbol().length > 0
        ? []
        : Object.entries(discoveryQuery.data?.options ?? {}).map(
            ([source, value]) => {
                if (value.symbol) {
                    return [value.symbol, source];
                }

                return null;
            })
            .filter(s => !!s)),
    );
    const decimalsSuggestions = createMemo(() => (assetDecimals() !== undefined && assetDecimals() !== 0
        ? []
        : Object.entries(discoveryQuery.data?.options ?? {}).map(
            ([source, value]) => {
                if (value.decimals) {
                    return [value.decimals, source];
                }

                return null;
            })
            .filter(s => !!s)),
    );
    const iconSuggestions = createMemo(() => (assetIconUrl().length > 0
        ? []
        : Object.entries(discoveryQuery.data?.options ?? {}).map(
            ([source, value]) => {
                console.log(source, value);

                if (value.icon_url) {
                    return [value.icon_url, source];
                }

                return null;
            })
            .filter(s => !!s)),
    );

    return (
        <>
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
                        <SegmentedControl.Control>
                            <SegmentedControl.Indicator />
                            <div class="flex gap-2 w-fit relative">
                                <For each={Object.keys(PLACEHOLDERS)}>
                                    {key => (
                                        <SegmentedControl.Item
                                          value={key}
                                        >
                                            <SegmentedControl.ItemInput class="" />
                                            <SegmentedControl.ItemLabel>
                                                {key}
                                            </SegmentedControl.ItemLabel>
                                        </SegmentedControl.Item>
                                    )}
                                </For>
                            </div>
                        </SegmentedControl.Control>
                    </SegmentedControl>
                    <Show when={assetType() === "erc20" || assetType() === "native"}>
                        <div class="space-y-1 block w-full">
                            <span>Network</span>
                            <NetworkSelect multiple={false} value={() => [networkId()]} onChange={x => (x ? x[0] && setNetworkId(x[0]) : setNetworkId(0))} />
                        </div>
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
                    <div class="flex justify-end text-end">
                        <ul>
                            <For each={nameSuggestions()}>
                                {([name]) => (
                                    <li>
                                        <button onClick={() => setAssetName(name)} class={button({ variant: "ghost", size: "small", class: "text-muted" })}>{name}</button>
                                    </li>
                                )}
                            </For>
                        </ul>
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
                            <ul class="flex justify-end text-end">
                                <For each={symbolSuggestions()}>
                                    {([symbol]) => (
                                        <li>
                                            <button onClick={() => setAssetSymbol(symbol)} class={button({ variant: "ghost", size: "small", class: "text-muted" })}>{symbol}</button>
                                        </li>
                                    )}
                                </For>
                            </ul>
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
                            <ul class="flex justify-end text-end">
                                <For each={decimalsSuggestions()}>
                                    {([decimals]) => (
                                        <li>
                                            <button onClick={() => setAssetDecimals(Number(decimals))} class={button({ variant: "ghost", size: "small", class: "text-muted" })}>{decimals}</button>
                                        </li>
                                    )}
                                </For>
                            </ul>
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
                    <ul class="flex justify-end text-end gap-2">
                        <For each={iconSuggestions()}>
                            {([iconUrl, source]) => (
                                <li>
                                    <button class={button({ variant: "outline" })} onClick={() => setAssetIconUrl(iconUrl)}>
                                        <CachedImage src={iconUrl} alt={source} class="size-8 aspect-square rounded-full" />
                                        {source}
                                    </button>
                                </li>
                            )}
                        </For>
                    </ul>
                </label>
            </div>
            <div class="w-full flex justify-end gap-2 p-4">
                <button
                  class={button({ variant: "primary" })}
                  onClick={() => assetCreate.mutate({ data: asset()! })}
                  disabled={!asset()}
                >
                    Create
                </button>
                <Modal.CloseButton class={button({ variant: "secondary" })}>
                    Cancel
                </Modal.CloseButton>
            </div>
        </>
    );
};

type AssetAddProps = {
    open?: boolean;
    onOpenChange?: (open: boolean) => void;
    onSuccess?: (asset: Asset) => void;
    children?: JSX.Element;
};

export const AssetAdd: Component<AssetAddProps> = (props) => {
    const [internalOpen, setInternalOpen] = createSignal(false);
    const open = () => props.open ?? internalOpen();
    const setOpen = (value: boolean) => {
        props.onOpenChange?.(value);

        if (props.open === undefined) {
            setInternalOpen(value);
        }
    };

    return (
        <Modal
          open={open()}
          onOpenChange={setOpen}
        >
            <Show when={props.children !== undefined || props.open === undefined}>
                <Modal.Trigger class={props.children ? "" : button({ variant: "primary", class: "text-sm" })}>
                    {props.children ?? (
                        <>
                            Add
                            {" "}
                            <FiPlus />
                        </>
                    )}
                </Modal.Trigger>
            </Show>
            <Modal.Portal>
                <Modal.Overlay />
                <Modal.Positioner>
                    <Modal.Content class="w-full max-w-xl bg-surface rounded-md relative mx-auto mt-24">
                        <Modal.CloseButton />
                        <AssetAddInner onClose={() => setOpen(false)} onSuccess={props.onSuccess} />
                    </Modal.Content>
                </Modal.Positioner>
            </Modal.Portal>
        </Modal>
    );
};
