import { FiPlus, FiX } from "solid-icons/fi";
import { Component, createMemo, createSignal, For, JSX, Show } from "solid-js";

import { useAccount, useAccountAssets, useAddAccountAsset, useRemoveAccountAsset } from "#/api/account";
import { Asset, useAssets } from "#/api/asset";
import { AssetPreview } from "#/components/asset/preview";
import { Modal } from "#/components/dialog";
import { button } from "#/components/input/button";

import { AccountAssetEntryBalance } from "./balance";

const matchesSearch = (asset: Asset, query: string) => {
    const normalized = query.trim().toLowerCase();

    if (!normalized) return true;

    return (
        asset.asset_name.toLowerCase().includes(normalized)
        || asset.asset_symbol.toLowerCase().includes(normalized)
        || asset.asset_identity.toLowerCase().includes(normalized)
    );
};

const isRelevantAsset = (asset: Asset, accountNetworks: number[] | undefined) => {
    if (asset.asset_identity.startsWith("fiat:")) return false;

    const network_identity = asset.asset_identity.split(":")[1];

    return accountNetworks?.includes(Number(network_identity)) ?? false;
};

type AccountAssetManageProps = {
    account_identity: number;
    open?: boolean;
    onOpenChange?: (open: boolean) => void;
    onAddAsset?: () => void;
    children?: JSX.Element;
};

export const AccountAssetManage: Component<AccountAssetManageProps> = (props) => {
    const { account_identity, onAddAsset } = props;
    const [internalOpen, setInternalOpen] = createSignal(false);
    const [search, setSearch] = createSignal("");

    const open = () => props.open ?? internalOpen();
    const setOpen = (value: boolean) => {
        props.onOpenChange?.(value);

        if (props.open === undefined) {
            setInternalOpen(value);
        }
    };

    const assetsQuery = useAssets();
    const accountQuery = useAccount(() => ({ path: { account_identity } }));
    const accountAssetsQuery = useAccountAssets(() => ({ path: { account_identity } }));
    const linkAsset = useAddAccountAsset<{ account_identity: number; asset_identity: string; }>(({ account_identity, asset_identity }) => ({ path: { account_identity, asset_identity } }));
    const unlinkAsset = useRemoveAccountAsset<{ account_identity: number; asset_identity: string; }>(({ account_identity, asset_identity }) => ({ path: { account_identity, asset_identity } }));

    const accountNetworks = createMemo(() => accountQuery.data?.networks);
    const enabledAssetIds = createMemo(() => new Set(accountAssetsQuery.data ?? []));

    const assets = createMemo(() => (assetsQuery.data?.assets ?? [])
        .filter(asset => isRelevantAsset(asset, accountNetworks()))
        .filter(asset => matchesSearch(asset, search()))
        .map(asset => ({
            asset,
            enabled: enabledAssetIds().has(asset.asset_identity),
        }))
        .toSorted((a, b) => {
            if (a.enabled !== b.enabled) return a.enabled ? -1 : 1;

            return a.asset.asset_name.localeCompare(b.asset.asset_name);
        }));

    return (
        <Modal
          open={open()}
          onOpenChange={setOpen}
        >
            <Modal.Trigger class={props.children ? "" : button({ variant: "outline", class: "text-sm" })}>
                {props.children ?? "Manage assets"}
            </Modal.Trigger>
            <Modal.Portal>
                <Modal.Overlay />
                <Modal.Positioner>
                    <Modal.Content class="w-full max-w-xl bg-surface rounded-md relative mx-auto mt-24">
                        <Modal.CloseButton />
                        <Modal.Title>
                            Manage assets
                        </Modal.Title>
                        <div class="p-4 space-y-4">
                            <input
                              type="text"
                              placeholder="Search assets..."
                              class="input w-full"
                              value={search()}
                              onInput={event => setSearch(event.currentTarget.value)}
                            />
                            <ul class="max-h-[60vh] overflow-y-auto space-y-1">
                                <Show when={assets().length > 0}>
                                    <For each={assets()}>
                                        {({ asset, enabled }) => (
                                            <li class="hover:bg-surface-alt p-2 rounded-md flex items-center justify-between gap-3">
                                                <div class="grow flex justify-between items-center gap-3">
                                                    <AssetPreview asset={asset} />
                                                    <AccountAssetEntryBalance
                                                      account_identity={account_identity}
                                                      asset={asset}
                                                      enabled={open()}
                                                    />
                                                </div>
                                                <Show
                                                  when={enabled}
                                                  fallback={(
                                                        <button
                                                          class={button({ variant: "primary", square: true })}
                                                          title="Enable asset"
                                                          onClick={() => linkAsset.mutate({ account_identity, asset_identity: asset.asset_identity })}
                                                        >
                                                            <FiPlus />
                                                        </button>
                                                    )}
                                                >
                                                    <button
                                                      class={button({ variant: "secondary", square: true })}
                                                      title="Disable asset"
                                                      onClick={() => unlinkAsset.mutate({ account_identity, asset_identity: asset.asset_identity })}
                                                    >
                                                        <FiX />
                                                    </button>
                                                </Show>
                                            </li>
                                        )}
                                    </For>
                                </Show>
                                <li>
                                    <button
                                      type="button"
                                      class="hover:bg-surface-alt p-2 rounded-md flex items-center gap-3 w-full text-muted cursor-pointer"
                                      onClick={() => {
                                            setOpen(false);
                                            onAddAsset?.();
                                        }}
                                    >
                                        <div class="size-8 rounded-full border border-dashed border-border flex items-center justify-center">
                                            <FiPlus class="size-4" />
                                        </div>
                                        Add a new asset
                                    </button>
                                </li>
                            </ul>
                        </div>
                    </Modal.Content>
                </Modal.Positioner>
            </Modal.Portal>
        </Modal>
    );
};
