import { useNavigate, useRouterState } from "@tanstack/solid-router";
import { Command } from "cmdk-solid";
import { FiArrowUpRight, FiChevronRight } from "solid-icons/fi";
import { createMemo, For, Show } from "solid-js";

import { useAccount, useAccountAssets } from "#/api/account";
import { useAssets } from "#/api/asset";

import { CommandMenuItem, PagedCommandGroupProperties } from "./item";

export const AssetCommands = (props: PagedCommandGroupProperties) => {
    const navigate = useNavigate();
    const routerState = useRouterState();
    const currentAccountId = createMemo(() => {
        const match = routerState().location.pathname.match(/^\/acc\/(\d+)/);

        return match?.[1] ? Number.parseInt(match[1]) : undefined;
    });
    const currentAccount = useAccount(() => ({
        path: { account_identity: currentAccountId() ?? 0 },
    }), { enabled: () => currentAccountId() !== undefined });
    const accountAssets = useAccountAssets(() => ({
        path: { account_identity: currentAccountId() ?? 0 },
    }), { enabled: () => currentAccountId() !== undefined });
    const assets = useAssets();
    const canSign = createMemo(
        () => currentAccount.data !== undefined && currentAccount.data.metadata.type !== "view",
    );
    const trackedAssets = createMemo(() => new Set(accountAssets.data ?? []));
    const sendableAssets = createMemo(
        () => assets.data?.assets
            .filter(asset => trackedAssets().has(asset.asset_identity))
            .toSorted((a, b) => a.asset_symbol.localeCompare(b.asset_symbol)) ?? [],
    );
    const showAssetOptions = () => props.page() === "assets"
      || (props.page() === undefined && props.search().length > 0);

    const sendAsset = (assetIdentity: string) => {
        const accountId = currentAccountId();

        if (accountId === undefined) return;

        props.close();
        navigate({
            to: `/acc/${accountId}/new-tx`,
            search: { type: "send", asset: assetIdentity },
        });
    };

    return (
        <Show when={canSign() && sendableAssets().length > 0}>
            <Show when={props.page() === undefined}>
                <Command.Group heading="Transactions">
                    <CommandMenuItem
                      value="send asset"
                      keywords={["token", "transfer", "choose asset"]}
                      icon={FiArrowUpRight}
                      title="Send asset..."
                      description={`${sendableAssets().length} tracked asset${sendableAssets().length === 1 ? "" : "s"}`}
                      suffix={<FiChevronRight class="size-4 text-muted" />}
                      onSelect={() => props.openPage("assets")}
                    />
                </Command.Group>
            </Show>
            <Show when={showAssetOptions()}>
                <Command.Group heading="Send asset">
                    <For each={sendableAssets()}>
                        {asset => (
                            <CommandMenuItem
                              value={`send asset ${asset.asset_symbol} ${asset.asset_identity}`}
                              keywords={["transfer", asset.asset_name, asset.asset_symbol, asset.asset_identity]}
                              icon={FiArrowUpRight}
                              title={`Send ${asset.asset_symbol}`}
                              description={asset.asset_name}
                              onSelect={() => sendAsset(asset.asset_identity)}
                            />
                        )}
                    </For>
                </Command.Group>
            </Show>
        </Show>
    );
};
