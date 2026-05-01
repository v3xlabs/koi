import { createEffect, createMemo, createSignal, Show } from "solid-js";
import { match } from "ts-pattern";

import { Asset, useCreateAsset } from "#/api/asset";

import { Modal } from "../dialog";
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
                            <div class="space-y-2">
                                <label class="space-y-1 block w-full">
                                    <span>Type</span>
                                    <select class="input w-full" value={assetType()} onChange={e => setAssetType(e.target.value)}>
                                        <option value="erc20">ERC20</option>
                                        <option value="native">Native</option>
                                        <option value="fiat">Fiat</option>
                                    </select>
                                </label>
                            </div>
                            <Show when={assetType() === "erc20" || assetType() === "native"}>
                                <label class="space-y-1 block w-full">
                                    <span>Network</span>
                                    {/* TODO: make single network select instead of this hack lmao */}
                                    <NetworkSelect value={() => [networkId()]} onChange={x => (x ? x[0] && setNetworkId(x.at(-1)!) : setNetworkId(0))} />
                                </label>
                            </Show>
                            <Show when={assetType() === "erc20"}>
                                <label class="space-y-1 block w-full">
                                    <span>Address</span>
                                    <input
                                      type="text"
                                      class="input w-full"
                                      value={assetAddress()}
                                      onChange={e => setAssetAddress(e.target.value)}
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
                            </label>
                            <label class="space-y-1 block w-full">
                                <span>Symbol</span>
                                <input
                                  type="text"
                                  class="input w-full"
                                  value={assetSymbol()}
                                  onChange={e => setAssetSymbol(e.target.value)}
                                  placeholder={PLACEHOLDERS[assetType()].symbol}
                                />
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
                            </label>
                            <label class="space-y-1 block w-full">
                                <span>Icon URL</span>
                                <input
                                  type="text"
                                  class="input w-full"
                                  value={assetIconUrl()}
                                  onChange={e => setAssetIconUrl(e.target.value)}
                                  placeholder={PLACEHOLDERS[assetType()].iconUrl}
                                />
                            </label>
                        </div>
                        <div>
                            <Show when={asset()}>
                                {asset => (
                                    <div>
                                        <div>
                                            {JSON.stringify(asset())}
                                        </div>
                                    </div>
                                )}
                            </Show>
                        </div>
                        <div class="w-full flex justify-end gap-2 p-4">
                            <button
                              class="btn btn-primary"
                              onClick={() => assetCreate.mutate({ data: asset() })}
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
