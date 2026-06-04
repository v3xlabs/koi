import { Toast, toaster } from "@kobalte/core/toast";
import { Link, useParams } from "@tanstack/solid-router";
import { FaSolidCopy, FaSolidExternalLink, FaSolidGear, FaSolidQrcode } from "solid-icons/fa";
import { FiLock, FiUnlock } from "solid-icons/fi";
import { For, Show, Suspense } from "solid-js";

import { useAccount } from "#/api/account";
import { usePrivacyMode } from "#/api/context";
import { truncateAddress } from "#/utils/address";
import { narrow } from "#/utils/narrow";
import { ReceiveQR } from "#/views/receive/qr";

import { AccountBalance } from "../account/balance";
import { AccountIcon } from "../account/icon";
import { AccountExternalLinkModal } from "../account/link";
import { AccountTypeIcon } from "../account/type";
import { ConnectionButton } from "../connection";
import { Modal } from "../dialog";
import { button } from "../input/button";
import { NetworkWidget } from "./networks";

export const Navbar = () => {
    const { privacyMode, setPrivacyMode } = usePrivacyMode();
    const params = useParams({ from: "/acc/$account" });
    const account_identity = Number.parseInt(params().account);
    const account = useAccount(() => ({ path: { account_identity } }));

    return (
        <div class="px-4 flex items-stretch justify-between w-full min-w-0 shrink-0 py-2 mt-1">
            <div class="flex min-w-0 items-center gap-2 bg-surface rounded-md px-3 py-1">
                <div class="flex items-center gap-2 pl-1 pr-2 py-2 w-full min-w-64">
                    <div class="size-9 bg-surface-alt rounded-md">
                        <Show when={narrow(() => account.data?.metadata, x => "evm_address" in x)}>
                            {acc => <AccountIcon address={() => acc().evm_address} />}
                        </Show>
                    </div>
                    <div class="leading-none grow">
                        <div class="font-medium text-sm leading-none flex items-center gap-1.5">
                            <span>
                                {account.data?.name}
                            </span>
                            <Show when={account.data?.metadata.type}>
                                {type => <AccountTypeIcon type={type} />}
                            </Show>
                        </div>
                        <Show when={narrow(() => account.data?.metadata, x => "evm_address" in x)}>
                            {acc => (
                                <div class="text-muted text-sm leading-none">
                                    {truncateAddress(acc().evm_address)}
                                </div>
                            )}
                        </Show>
                    </div>
                    <div class="text-sm leading-none">
                        <Suspense>
                            <AccountBalance account_identity={account_identity} />
                        </Suspense>
                    </div>
                </div>
                <div class="flex gap-2 px-1">
                    <Show when={narrow(() => account.data?.metadata, x => "evm_address" in x)}>
                        {acc => (
                            <ReceiveQR address={() => acc().evm_address}>
                                <Modal.Trigger class={button({ variant: "secondary", square: true })}>
                                    <FaSolidQrcode class="w-3.5 h-3.5 text-secondary-foreground" />
                                </Modal.Trigger>
                            </ReceiveQR>
                        )}
                    </Show>
                    <Show when={narrow(() => account.data?.metadata, x => "evm_address" in x)}>
                        {acc => (
                            <button
                              class={button({ variant: "secondary", square: true })}
                              onClick={async () => {
                                    try {
                                        await navigator.clipboard.writeText(acc().evm_address);
                                        toaster.show(props => (
                                            <Toast toastId={props.toastId} class="toast">
                                                <div class="flex justify-between items-center">
                                                    <div>Address copied</div>
                                                </div>
                                            </Toast>
                                        ));
                                    }
                                    catch {
                                        toaster.show(props => (
                                            <Toast toastId={props.toastId} class="toast">
                                                <div class="flex justify-between items-center">
                                                    <div>Failed to copy address</div>
                                                </div>
                                            </Toast>
                                        ));
                                    }
                                }}
                            >
                                <FaSolidCopy />
                            </button>
                        )}
                    </Show>
                    <AccountExternalLinkModal account_identity={account_identity} class={button({ variant: "secondary", square: true })}>
                        <FaSolidExternalLink />
                    </AccountExternalLinkModal>
                    <For each={[
                        {
                            icon: FaSolidGear,
                            label: "Settings",
                            href: "/acc/$account/settings",
                        },
                    ]}
                    >
                        {item => (
                            <Link
                              to={item.href}
                              class={button({ variant: "secondary", square: true })}
                            >
                                <item.icon class="w-3.5 h-3.5 text-secondary-foreground" />
                            </Link>
                        )}
                    </For>
                </div>
            </div>
            <div class="min-w-0">
            </div>
            <div class="flex shrink-0 items-stretch self-stretch gap-2">
                <ConnectionButton />
                <NetworkWidget />
                <button
                  class="nav-icon-button"
                  type="button"
                  aria-label={privacyMode() ? "Disable privacy mode" : "Enable privacy mode"}
                  aria-pressed={privacyMode()}
                  title={privacyMode() ? "Disable privacy mode" : "Enable privacy mode"}
                  onClick={() => setPrivacyMode(!privacyMode())}
                >
                    {privacyMode() ? <FiLock /> : <FiUnlock />}
                </button>
            </div>
        </div>
    );
};
