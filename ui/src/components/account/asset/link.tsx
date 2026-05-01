import { FiPlus, FiRefreshCcw, FiX } from "solid-icons/fi";
import { Component, createMemo, For } from "solid-js";

import { useAccount, useAccountAssets, useAddAccountAsset, useRemoveAccountAsset } from "#/api/account";
import { useAssets } from "#/api/asset";
import { AssetPreview } from "#/components/asset/preview";
import { Modal } from "#/components/dialog";

export const AccountAssetLink: Component<{ account_identity: number; }> = ({ account_identity }) => {
    const assetsQuery = useAssets();
    const accountQuery = useAccount(() => ({ path: { account_identity } }));
    const accountAssetsQuery = useAccountAssets(() => ({ path: { account_identity } }));
    const linkAsset = useAddAccountAsset<{ account_identity: number; asset_identity: string; }>(({ account_identity, asset_identity }) => ({ path: { account_identity, asset_identity } }));

    const account = createMemo(() => accountQuery.data);

    // Filters out assets already linked, and fiat assets (as linking a fiat asset to an evm wallet should not be done)
    const filteredAssets = createMemo(() => assetsQuery.data?.assets?.filter(asset => !asset.asset_identity.startsWith("fiat:") && !accountAssetsQuery.data?.includes(asset.asset_identity)).filter((asset) => {
        const network_identity = asset.asset_identity.split(":")[1];

        return account()?.networks.includes(Number(network_identity));
    }));

    return (
        <Modal>
            <Modal.Trigger class="btn btn-primary">
                Add Assets
            </Modal.Trigger>
            <Modal.Portal>
                <Modal.Overlay />
                <div class="fixed inset-0">
                    <Modal.Content class="w-full max-w-xl bg-surface rounded-md relative mx-auto mt-24">
                        <Modal.Title>
                            Link Assets
                        </Modal.Title>
                        <div class="p-4 space-y-4">
                            <div class="w-full">
                                <input type="text" placeholder="Search for a token" class="input w-full" />
                            </div>
                            <div>
                                <ul>
                                    <For each={filteredAssets()} fallback={<div class="text-center text-muted">No assets remain</div>}>
                                        {asset => (
                                            <li class="hover:bg-surface-alt cursor-pointer p-2 rounded-md flex items-center justify-between">
                                                <AssetPreview asset_identity={asset.asset_identity} />
                                                <div class="flex items-center gap-2">
                                                    <button class="btn btn-secondary aspect-square flex items-center justify-center" disabled>
                                                        <FiRefreshCcw />
                                                    </button>
                                                    <button
                                                      class="btn btn-primary aspect-square flex items-center justify-center"
                                                      onClick={() => linkAsset.mutate({ account_identity, asset_identity: asset.asset_identity })}
                                                    >
                                                        <FiPlus />
                                                    </button>
                                                </div>
                                            </li>
                                        )}
                                    </For>
                                </ul>
                            </div>
                        </div>
                    </Modal.Content>
                </div>
            </Modal.Portal>
        </Modal>
    );
};

export const AssetUnlink: Component<{ account_identity: number; asset_identity: string; }> = ({ account_identity, asset_identity }) => {
    const unlinkAsset = useRemoveAccountAsset<{ account_identity: number; asset_identity: string; }>(({ account_identity, asset_identity }) => ({ path: { account_identity, asset_identity } }));

    return (
        <button class="btn btn-secondary aspect-square flex items-center justify-center" onClick={() => unlinkAsset.mutate({ account_identity, asset_identity })}>
            <FiX />
        </button>
    );
};
