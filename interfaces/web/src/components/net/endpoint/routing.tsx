import { Popover } from "@kobalte/core/popover";
import { Tooltip } from "@kobalte/core/tooltip";
import { FiMoreHorizontal } from "solid-icons/fi";
import type { Component } from "solid-js";

import { button } from "#/components/input/button";
import { Toggle } from "#/components/input/toggle";

const unavailableRouting = () => undefined;

const UnavailableRoutingOption: Component<{ label: string; }> = props => (
    <Tooltip>
        <Tooltip.Trigger as="span" class="block">
            <Toggle
              value={() => false}
              onChange={unavailableRouting}
              label={props.label}
              disabled
            />
        </Tooltip.Trigger>
        <Tooltip.Portal>
            <Tooltip.Content class="bg-surface-alt text-secondary-foreground rounded-md border border-border p-2 text-xs">
                <Tooltip.Arrow />
                Coming soon
            </Tooltip.Content>
        </Tooltip.Portal>
    </Tooltip>
);

export const NetworkEndpointRoutingMenu: Component = () => (
    <Popover>
        <Popover.Trigger class={button({ variant: "ghost", size: "default", square: true })} aria-label="Routing options">
            <FiMoreHorizontal />
        </Popover.Trigger>
        <Popover.Portal>
            <Popover.Content class="popover-content w-64 space-y-3 p-4">
                <div class="text-sm">Routing</div>
                <UnavailableRoutingOption label="Tor" />
                <UnavailableRoutingOption label="Proxy" />
            </Popover.Content>
        </Popover.Portal>
    </Popover>
);
