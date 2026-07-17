import { useNavigate } from "@tanstack/solid-router";
import { Command } from "cmdk-solid";
import {
    FaSolidAddressCard,
    FaSolidCalculator,
    FaSolidCoins,
    FaSolidGear,
    FaSolidHandshake,
    FaSolidNetworkWired,
} from "solid-icons/fa";
import { For } from "solid-js";

import { CommandGroupProperties, CommandMenuItem } from "./item";

const destinations = [
    {
        value: "go accounts",
        keywords: ["home", "wallets"],
        icon: FaSolidAddressCard,
        title: "Accounts",
        description: "Open the account list",
        to: "/",
    },
    {
        value: "go address book",
        keywords: ["contacts"],
        icon: FaSolidAddressCard,
        title: "Address book",
        description: "Open saved contacts",
        to: "/addressbook",
    },
    {
        value: "go settings",
        keywords: ["preferences"],
        icon: FaSolidGear,
        title: "Settings",
        description: "Open app settings",
        to: "/settings",
    },
    {
        value: "go networks",
        keywords: ["chains", "rpc", "endpoints"],
        icon: FaSolidNetworkWired,
        title: "Networks",
        description: "Manage chains and endpoints",
        to: "/settings/networks",
    },
    {
        value: "go assets",
        keywords: ["tokens"],
        icon: FaSolidCoins,
        title: "Assets",
        description: "Manage known assets",
        to: "/settings/assets",
    },
    {
        value: "go price feeds",
        keywords: ["quoters", "quotes", "prices"],
        icon: FaSolidCalculator,
        title: "Price feeds",
        description: "Manage asset price providers",
        to: "/settings/quoters",
    },
    {
        value: "go vendors",
        keywords: ["providers", "integrations"],
        icon: FaSolidHandshake,
        title: "Vendors",
        description: "Configure external data providers",
        to: "/settings/vendors",
    },
] as const;

export const NavigationCommands = (props: CommandGroupProperties) => {
    const navigate = useNavigate();

    const go = (to: string) => {
        props.close();
        navigate({ to });
    };

    return (
        <Command.Group heading="Navigate">
            <For each={destinations}>
                {destination => (
                    <CommandMenuItem
                      value={destination.value}
                      keywords={[...destination.keywords]}
                      icon={destination.icon}
                      title={destination.title}
                      description={destination.description}
                      onSelect={() => go(destination.to)}
                    />
                )}
            </For>
        </Command.Group>
    );
};
