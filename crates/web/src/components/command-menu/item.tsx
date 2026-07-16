import { Command } from "cmdk-solid";
import { Accessor, Component, JSX, Show } from "solid-js";

export type CommandPage = "accounts" | "assets" | "currency" | "theme";

export type CommandGroupProperties = {
    close: () => void;
};

export type PagedCommandGroupProperties = CommandGroupProperties & {
    page: Accessor<CommandPage | undefined>;
    search: Accessor<string>;
    openPage: (page: CommandPage) => void;
};

export const CommandMenuItem: Component<{
    value: string;
    keywords?: string[];
    icon: Component<{ class?: string; }>;
    title: string;
    description?: string;
    suffix?: JSX.Element;
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
        {props.suffix}
    </Command.Item>
);
