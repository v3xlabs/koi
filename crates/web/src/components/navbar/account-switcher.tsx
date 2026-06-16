import { Popover } from "@kobalte/core/popover";
import { Toast, toaster } from "@kobalte/core/toast";
import { Link, useNavigate, useParams, useRouterState } from "@tanstack/solid-router";
import { FaSolidCopy, FaSolidExternalLink, FaSolidGear, FaSolidQrcode } from "solid-icons/fa";
import { FiChevronDown } from "solid-icons/fi";
import { createSignal, For, Show } from "solid-js";

import { useAccount, useAccountLayout } from "#/api/account";
import { narrow } from "#/utils/narrow";
import { ReceiveQR } from "#/views/receive/qr";

import { AccountExternalLinkModal } from "../account/link";
import { AccountSelectorList } from "../account/selector-list";
import { AccountSwitcherItem } from "../account/switcher-item";
import { Modal } from "../dialog";

const TRIGGER_OVERLAP = 52;

const switcherTriggerClass = "flex w-full cursor-pointer items-center gap-2 rounded-md bg-surface px-2 h-10 text-left transition-colors hover:bg-surface-alt hover:border border-border data-[expanded]:opacity-0";

export const AccountSwitcher = () => {
    const navigate = useNavigate();
    const routerState = useRouterState();
    const params = useParams({ from: "/acc/$account" });
    const account_identity = () => Number.parseInt(params().account);
    const layoutQuery = useAccountLayout();
    const [open, setOpen] = createSignal(false);

    const switchAccount = (nextAccountId: number) => {
        if (nextAccountId === account_identity()) {
            setOpen(false);

            return;
        }

        const nextPath = routerState().location.pathname.replace(/^\/acc\/\d+/, `/acc/${nextAccountId}`);

        navigate({ to: nextPath });
        setOpen(false);
    };

    return (
        <Popover
          open={open()}
          onOpenChange={setOpen}
          placement="bottom-start"
          gutter={-TRIGGER_OVERLAP}
          shift={0}
          flip={false}
          sameWidth
        >
            <Popover.Trigger class={switcherTriggerClass}>
                <div class="w-full flex-1">
                    <AccountSwitcherItem account_identity={account_identity()} />
                </div>
            </Popover.Trigger>
            <Popover.Portal>
                <Popover.Content class="account-switcher-popover popover-content z-50 overflow-hidden p-0">
                    <AccountSelectorList
                      open={open()}
                      layout={layoutQuery.data}
                      activeAccountId={account_identity()}
                      onSelect={switchAccount}
                    />
                </Popover.Content>
            </Popover.Portal>
        </Popover>
    );
};

export const AccountNavbarActions = () => {
    const params = useParams({ from: "/acc/$account" });
    const account_identity = () => Number.parseInt(params().account);
    const account = useAccount(() => ({ path: { account_identity: account_identity() } }));

    return (
        <div class="flex shrink-0 gap-2 self-stretch">
            <Show when={narrow(() => account.data?.metadata, x => "evm_address" in x)}>
                {acc => (
                    <ReceiveQR address={() => acc().evm_address}>
                        <Modal.Trigger class="nav-icon-button">
                            <FaSolidQrcode />
                        </Modal.Trigger>
                    </ReceiveQR>
                )}
            </Show>
            <Show when={narrow(() => account.data?.metadata, x => "evm_address" in x)}>
                {acc => (
                    <button
                      type="button"
                      class="nav-icon-button"
                      onClick={async () => {
                            try {
                                await navigator.clipboard.writeText(acc().evm_address);
                                toaster.show(props => (
                                    <Toast toastId={props.toastId} class="toast">
                                        <div class="flex items-center justify-between">
                                            <div>Address copied</div>
                                        </div>
                                    </Toast>
                                ));
                            }
                            catch {
                                toaster.show(props => (
                                    <Toast toastId={props.toastId} class="toast">
                                        <div class="flex items-center justify-between">
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
            <AccountExternalLinkModal account_identity={account_identity()} class="nav-icon-button">
                <FaSolidExternalLink />
            </AccountExternalLinkModal>
            <For each={[
                {
                    icon: FaSolidGear,
                    href: "/acc/$account/settings",
                },
            ]}
            >
                {item => (
                    <Link
                      to={item.href}
                      params={{ account: params().account }}
                      class="nav-icon-button"
                    >
                        <item.icon />
                    </Link>
                )}
            </For>
        </div>
    );
};
