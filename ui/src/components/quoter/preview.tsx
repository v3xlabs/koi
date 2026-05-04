import { Component, Show, Suspense } from "solid-js";

import { useQuoter } from "#/api/quoter";

import { AssetPreview } from "../asset/preview";

export const QuoterPreview: Component<{ quoter_identity: string; }> = ({ quoter_identity }) => {
    const quoterQuery = useQuoter(() => ({ path: { quoter_identity } }));

    return (
        <div class="w-full flex items-center gap-2">
            <div class="block">
                <div>
                    {quoterQuery.data?.quoter_name}
                </div>
                <span class="text-muted text-sm">
                    #
                    {quoterQuery.data?.quoter_identity}
                </span>
            </div>
            <span class="text-muted text-sm">
                <Show when={quoterQuery.data?.token_a}>
                    {token_a => (
                        <Suspense fallback={<div>Loading...</div>}>
                            <AssetPreview asset_identity={token_a()} />
                        </Suspense>
                    )}
                </Show>
            </span>
            <span class="text-muted text-sm">
                <Show when={quoterQuery.data?.token_b}>
                    {token_b => (
                        <Suspense fallback={<div>Loading...</div>}>
                            <AssetPreview asset_identity={token_b()} />
                        </Suspense>
                    )}
                </Show>
            </span>
        </div>
    );
};
