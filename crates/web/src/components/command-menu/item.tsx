import { Command } from "cmdk-solid";
import { Component, Show } from "solid-js";

export type CommandGroupProperties = {
    close: () => void;
};

export const CommandMenuItem: Component<{
    value: string;
    keywords?: string[];
    icon: Component<{ class?: string; }>;
    title: string;
    description?: string;
    onSelect: () => void;
}> = props => (
    <Command.Item
      value={props.value}
      keywords={props.keywords}
      onSelect={props.onSelect}
      class="command-menu__item"
    >
        <div class="command-menu__item-icon">
            <props.icon class="size-4" />
        </div>
        <div class="min-w-0 flex-1">
            <div class="truncate text-sm font-medium">{props.title}</div>
            <Show when={props.description}>
                {description => <div class="truncate text-xs text-muted">{description()}</div>}
            </Show>
        </div>
    </Command.Item>
);
