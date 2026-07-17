import { Toast, toaster } from "@kobalte/core/toast";
import { useNavigate, useRouterState } from "@tanstack/solid-router";
import { Command } from "cmdk-solid";
import { FaSolidEye } from "solid-icons/fa";
import { FiCopy, FiSend } from "solid-icons/fi";
import { Accessor, createMemo, Show } from "solid-js";
import { isAddress } from "viem";

import { useAccount } from "#/api/account";
import { parseAddressInput } from "#/utils/address";

import { CommandGroupProperties, CommandMenuItem } from "./item";

type ContextualCommandProperties = CommandGroupProperties & {
    search: Accessor<string>;
};

const showToast = (message: string) => toaster.show(props => (
    <Toast toastId={props.toastId} class="toast">
        <div>{message}</div>
    </Toast>
));

export const ContextualCommands = (props: ContextualCommandProperties) => {
    const navigate = useNavigate();
    const routerState = useRouterState();
    const currentAccountId = createMemo(() => {
        const match = routerState().location.pathname.match(/^\/acc\/(\d+)/);

        return match?.[1] ? Number.parseInt(match[1]) : undefined;
    });
    const currentAccount = useAccount(() => ({
        path: { account_identity: currentAccountId() ?? 0 },
    }), { enabled: () => currentAccountId() !== undefined });
    const canSign = createMemo(
        () => currentAccount.data !== undefined && currentAccount.data.metadata.type !== "view",
    );
    const address = createMemo(() => {
        const parsed = parseAddressInput(props.search()).address;

        return isAddress(parsed) ? parsed : undefined;
    });

    const sendToAddress = (address: string) => {
        const accountId = currentAccountId();

        if (accountId === undefined) return;

        props.close();
        navigate({
            to: `/acc/${accountId}/new-tx`,
            search: { type: "send", to: address },
        });
    };

    const importAddress = (address: string) => {
        props.close();
        navigate({
            to: "/acc/import/view",
            search: { address },
        });
    };

    const copyAddress = (address: string) => {
        props.close();
        void navigator.clipboard.writeText(address).then(
            () => showToast("Address copied"),
            () => showToast("Failed to copy address"),
        );
    };

    return (
        <Show when={address()}>
            {address => (
                <Command.Group heading="Address actions">
                    <Show when={canSign()}>
                        <CommandMenuItem
                          value={`send to address ${props.search()} ${address()}`}
                          keywords={["transfer", "transaction", address()]}
                          icon={FiSend}
                          title="Send to address"
                          description={address()}
                          onSelect={() => sendToAddress(address())}
                        />
                    </Show>
                    <CommandMenuItem
                      value={`import view only address ${props.search()} ${address()}`}
                      keywords={["watch", "account", address()]}
                      icon={FaSolidEye}
                      title="Import as view-only account"
                      description={address()}
                      onSelect={() => importAddress(address())}
                    />
                    <CommandMenuItem
                      value={`copy address ${props.search()} ${address()}`}
                      keywords={["clipboard", address()]}
                      icon={FiCopy}
                      title="Copy address"
                      description={address()}
                      onSelect={() => copyAddress(address())}
                    />
                </Command.Group>
            )}
        </Show>
    );
};
