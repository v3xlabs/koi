import { FiPlus, FiRefreshCcw } from "solid-icons/fi";
import { Component, createMemo, For } from "solid-js";

import { useAccountAssets, useAddAccountAsset } from "#/api/account";
import { useAssets } from "#/api/asset";
import { AssetPreview } from "#/components/asset/preview";
import { Modal } from "#/components/dialog";

export const AccountAssetLink: Component<{ account_identity: number; }> = ({ account_identity }) => {
    const assetsQuery = useAssets();
    const accountAssetsQuery = useAccountAssets(() => ({ path: { account_identity } }));
    const linkAsset = useAddAccountAsset<{ asset_identity: string; }>(({ asset_identity }) => ({ path: { account_identity, asset_identity } }), {
        onSuccess(data, variables, onMutateResult, context) {
            console.log("f", data, variables, onMutateResult, context);
            context.client.invalidateQueries({ queryKey: ["account", account_identity, "assets"] });
        },
    });

    // Filters out assets already linked, and fiat assets (as linking a fiat asset to an evm wallet should not be done)
    const filteredAssets = createMemo(() => assetsQuery.data?.assets?.filter(asset => !asset.asset_identity.startsWith("fiat:") && !accountAssetsQuery.data?.includes(asset.asset_identity)));

    return (
        <Modal>
            <Modal.Trigger class="btn btn-primary">
                Add tokens
            </Modal.Trigger>
            <Modal.Portal>
                <Modal.Overlay />
                <div class="fixed inset-0">
                    <Modal.Content class="w-full max-w-xl bg-surface rounded-md relative mx-auto mt-24">
                        <Modal.Title>
                            Link Asset
                        </Modal.Title>
                        <div>
                            <div>
                                <input type="text" placeholder="Search for a token" />
                            </div>
                            <div>
                                <ul>
                                    <For each={filteredAssets()} fallback={<div>No assets remain</div>}>
                                        {asset => (
                                            <li class="hover:bg-surface-alt cursor-pointer p-2 rounded-md flex items-center justify-between">
                                                <AssetPreview asset_identity={asset.asset_identity} />
                                                <div class="flex items-center gap-2">
                                                    <button class="btn btn-secondary aspect-square flex items-center justify-center" disabled>
                                                        <FiRefreshCcw />
                                                    </button>
                                                    <button
                                                      class="btn btn-primary aspect-square flex items-center justify-center"
                                                      onClick={() => linkAsset.mutate({ asset_identity: asset.asset_identity })}
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
