import { Command } from "cmdk-solid";
import { FiChevronRight, FiDollarSign, FiEye, FiEyeOff, FiMonitor, FiMoon, FiSun } from "solid-icons/fi";
import { createMemo, For, Show } from "solid-js";

import { useAssets } from "#/api/asset";
import { useDisplayCurrency, usePrivacyMode, useTheme } from "#/api/context";

import { CommandMenuItem, PagedCommandGroupProperties } from "./item";

const themes = [
    {
        value: "light" as const,
        icon: FiSun,
        title: "Use light theme",
        description: "Switch the interface to light mode",
    },
    {
        value: "dark" as const,
        icon: FiMoon,
        title: "Use dark theme",
        description: "Switch the interface to dark mode",
    },
    {
        value: "system" as const,
        icon: FiMonitor,
        title: "Use system theme",
        description: "Follow the operating system appearance",
    },
];

export const PreferenceCommands = (props: PagedCommandGroupProperties) => {
    const assets = useAssets();
    const { displayCurrency, setDisplayCurrency } = useDisplayCurrency();
    const { privacyMode, setPrivacyMode } = usePrivacyMode();
    const { theme, setTheme } = useTheme();
    const fiatAssets = createMemo(
        () => assets.data?.assets.filter(asset => asset.asset_identity.startsWith("fiat:")) ?? [],
    );
    const currentCurrency = createMemo(
        () => fiatAssets().find(asset => asset.asset_identity === displayCurrency()),
    );
    const searchingRoot = () => props.page() === undefined && props.search().length > 0;

    const run = (action: () => void) => {
        action();
        props.close();
    };

    return (
        <>
            <Show when={props.page() === undefined}>
                <Command.Group heading="Preferences">
                    <CommandMenuItem
                      value="toggle privacy mode"
                      keywords={["hide balances", "private", "sensitive values"]}
                      icon={privacyMode() ? FiEye : FiEyeOff}
                      title={privacyMode() ? "Disable privacy mode" : "Enable privacy mode"}
                      description="Hide or reveal sensitive wallet values"
                      onSelect={() => run(() => setPrivacyMode(!privacyMode()))}
                    />
                    <CommandMenuItem
                      value="change theme"
                      keywords={["appearance", "light", "dark", "system"]}
                      icon={themes.find(option => option.value === theme())?.icon ?? FiMonitor}
                      title="Change theme..."
                      description={`Current: ${theme()}`}
                      suffix={<FiChevronRight class="size-4 text-muted" />}
                      onSelect={() => props.openPage("theme")}
                    />
                    <Show when={fiatAssets().length > 0}>
                        <CommandMenuItem
                          value="change display currency"
                          keywords={["fiat", "quote", "money"]}
                          icon={FiDollarSign}
                          title="Change display currency..."
                          description={`Current: ${currentCurrency()?.asset_symbol ?? displayCurrency()}`}
                          suffix={<FiChevronRight class="size-4 text-muted" />}
                          onSelect={() => props.openPage("currency")}
                        />
                    </Show>
                </Command.Group>
            </Show>
            <Show when={props.page() === "theme" || searchingRoot()}>
                <Command.Group heading="Theme">
                    <For each={themes}>
                        {option => (
                            <CommandMenuItem
                              value={`set ${option.value} theme`}
                              keywords={["appearance", option.value]}
                              icon={option.icon}
                              title={option.title}
                              description={theme() === option.value ? "Currently active" : option.description}
                              onSelect={() => run(() => setTheme(option.value))}
                            />
                        )}
                    </For>
                </Command.Group>
            </Show>
            <Show when={props.page() === "currency" || searchingRoot()}>
                <Command.Group heading="Display currency">
                    <For each={fiatAssets()}>
                        {asset => (
                            <CommandMenuItem
                              value={`set display currency ${asset.asset_identity}`}
                              keywords={["fiat", "quote", asset.asset_name, asset.asset_symbol]}
                              icon={FiDollarSign}
                              title={`Use ${asset.asset_name}`}
                              description={displayCurrency() === asset.asset_identity
                                  ? "Currently active"
                                  : `Display wallet values in ${asset.asset_symbol}`}
                              onSelect={() => run(() => setDisplayCurrency(asset.asset_identity))}
                            />
                        )}
                    </For>
                </Command.Group>
            </Show>
        </>
    );
};
