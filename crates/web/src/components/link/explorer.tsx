import { FiExternalLink } from "solid-icons/fi";
import { Component, For, JSX, Show } from "solid-js";

import { useNetwork } from "#/api/network";
import { VendorFlag } from "#/api/vendor";
import { Modal } from "#/components/dialog";
import { button } from "#/components/input/button";
import { capitalFirst } from "#/utils/text";

import { NetworkIcon } from "../net/icon";
import { ExplorerIcon, ExplorerKey } from "./explorer-icon";

export const explorerNameFromFlag = (flag: VendorFlag) => capitalFirst(flag.split("_")[0]);

export const explorerKeyFromFlag = (flag: VendorFlag): ExplorerKey | null => {
    if (flag.startsWith("etherscan_")) return "etherscan";

    if (flag.startsWith("blockscout_")) return "blockscout";

    return null;
};

export type ExplorerLink = {
    link: string;
    explorerName: string;
    explorerKey: ExplorerKey;
    network_identity: number;
};

const NetworkLabel: Component<{ network_identity: number; }> = (props) => {
    const networkQuery = useNetwork(() => ({ path: { network_identity: props.network_identity } }));

    return (
        <span class="text-sm text-muted truncate">
            {networkQuery.data?.network_name ?? `Network ${props.network_identity}`}
        </span>
    );
};

export const ExplorerLinkCard: Component<ExplorerLink> = props => (
    <a
      href={props.link}
      target="_blank"
      rel="noopener noreferrer"
      class="group block rounded-lg border border-border p-3 transition-colors hover:border-primary/40 hover:bg-surface-alt"
    >
        <div class="mb-2 flex items-start justify-between gap-3">
            <div class="flex min-w-0 items-start gap-3">
                <div class="flex size-8 shrink-0 items-center justify-center">
                    <ExplorerIcon explorer={props.explorerKey} class="size-full object-contain" />
                </div>
                <div class="min-w-0 space-y-1">
                    <div class="text-sm font-medium">{props.explorerName}</div>
                    <div class="flex min-w-0 items-center gap-1.5 text-muted">
                        <NetworkIcon network_identity={props.network_identity} />
                        <NetworkLabel network_identity={props.network_identity} />
                    </div>
                </div>
            </div>
            <FiExternalLink class="mt-0.5 size-4 shrink-0 text-muted transition-colors group-hover:text-foreground" />
        </div>
        <code class="block break-all rounded-md bg-surface-alt/60 px-2.5 py-2 text-xs leading-relaxed text-muted">
            {props.link}
        </code>
    </a>
);

type ExplorerLinksModalProps = {
    title: string;
    description: string;
    children?: JSX.Element;
    class?: string;
    emptyMessage?: string;
    links: () => ExplorerLink[];
    open?: boolean;
    onOpenChange?: (open: boolean) => void;
};

export const ExplorerLinksModal: Component<ExplorerLinksModalProps> = props => (
    <Modal open={props.open} onOpenChange={props.onOpenChange}>
        <Show when={props.children}>
            {children => (
                <Modal.Trigger class={props.class}>
                    {children()}
                </Modal.Trigger>
            )}
        </Show>
        <Modal.Portal>
            <Modal.Overlay />
            <Modal.Positioner>
                <Modal.Content class="relative mx-auto mt-24 w-full max-w-lg rounded-md bg-surface">
                    <Modal.CloseButton />
                    <Modal.Title>{props.title}</Modal.Title>
                    <div class="space-y-4 p-4">
                        <p class="text-sm text-muted">
                            {props.description}
                        </p>
                        <Show
                          when={props.links().length > 0}
                          fallback={(
                                <p class="rounded-lg border border-dashed border-border px-3 py-6 text-center text-sm text-muted">
                                    {props.emptyMessage ?? "No block explorer links are available."}
                                </p>
                            )}
                        >
                            <ul class="max-h-[50vh] space-y-2 overflow-y-auto">
                                <For each={props.links()}>
                                    {link => (
                                        <li>
                                            <ExplorerLinkCard
                                              link={link.link}
                                              explorerName={link.explorerName}
                                              explorerKey={link.explorerKey}
                                              network_identity={link.network_identity}
                                            />
                                        </li>
                                    )}
                                </For>
                            </ul>
                        </Show>
                        <div class="flex justify-end pt-1">
                            <Modal.CloseButton class={button({ variant: "secondary" })}>
                                Cancel
                            </Modal.CloseButton>
                        </div>
                    </div>
                </Modal.Content>
            </Modal.Positioner>
        </Modal.Portal>
    </Modal>
);
