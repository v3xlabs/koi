import { SegmentedControl as KobalteSegmentedControl } from "@kobalte/core/segmented-control";
import { Component, JSX } from "solid-js";

type SegmentedControlRoot = typeof KobalteSegmentedControl;

// @ts-expect-error composite type
const Root: SegmentedControlRoot = props => (
    <KobalteSegmentedControl {...props} />
) as unknown as SegmentedControlRoot;

const Control: Component<JSX.HTMLAttributes<HTMLDivElement>> = props => (
    <div
      {...props}
      classList={{
            [props.class ?? ""]: !!props.class,
            "relative inline-flex rounded-md border border-border bg-surface p-1": true,
        }}
      role={props.role ?? "presentation"}
    />
);

const Indicator: typeof KobalteSegmentedControl.Indicator = props => (
    <KobalteSegmentedControl.Indicator
      {...props}
      classList={{
            [props.class ?? ""]: !!props.class,
            "absolute top-1 left-1 h-[calc(100%-0.5rem)] rounded-sm bg-primary transition-all duration-300": true,
        }}
    />
);

const Item: typeof KobalteSegmentedControl.Item = props => (
    <KobalteSegmentedControl.Item
      {...props}
      classList={{
            [props.class ?? ""]: !!props.class,
            "relative rounded-sm px-2 py-1 text-sm text-muted outline-none transition-colors data-[checked]:text-primary-foreground focus-visible:ring-2 focus-visible:ring-primary/50": true,
        }}
    />
);

const ItemLabel: typeof KobalteSegmentedControl.ItemLabel = props => (
    <KobalteSegmentedControl.ItemLabel
      {...props}
      classList={{
            [props.class ?? ""]: !!props.class,
            "cursor-pointer select-none": true,
        }}
    />
);

export const SegmentedControl = Object.assign(Root, {
    ...KobalteSegmentedControl,
    Control,
    Indicator,
    Item,
    ItemLabel,
});
