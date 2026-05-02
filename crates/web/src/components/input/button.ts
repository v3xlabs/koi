type ButtonVariant = "primary" | "secondary" | "tertiary" | "danger" | "ghost" | "outline";
type ButtonSize = "small" | "default" | "large";

type ButtonOptions = {
    variant?: ButtonVariant;
    size?: ButtonSize;
    square?: boolean;
    class?: string;
};

export const cn = (...classes: (false | null | string | undefined)[]) => classes.filter(Boolean).join(" ");

export const button = (options: ButtonOptions = {}) => {
    const variant = options.variant ?? "primary";
    const size = options.size ?? "default";

    return cn(
        "inline-flex items-center justify-center gap-2 whitespace-nowrap border font-medium leading-normal outline-none transition duration-75 select-none",
        "focus-visible:ring-2 focus-visible:ring-primary/50 focus-visible:ring-offset-2 focus-visible:ring-offset-background",
        "disabled:cursor-not-allowed not-disabled:cursor-pointer",
        "data-[selected]:border-primary data-[selected]:bg-surface-alt data-[selected]:text-foreground data-[pressed]:border-primary",
        "data-[pressed]:bg-surface-alt data-[pressed]:text-foreground",
        "aria-selected:border-primary aria-selected:bg-surface-alt aria-selected:text-foreground",
        "active:not-disabled:scale-95",
        size === "large" && (options.square ? "size-10 rounded-md" : "min-h-10 px-5 py-2 rounded-md text-base"),
        size === "default" && (options.square ? "size-9 rounded-md" : "min-h-9 px-3.5 py-1.5 rounded-md text-sm"),
        size === "small" && (options.square ? "size-7 rounded-md" : "min-h-7 px-2.5 py-1 rounded-md text-xs"),
        variant === "primary" && "bg-primary hover:bg-primary-hover text-primary-foreground disabled:bg-primary/50 disabled:text-primary-foreground/60 border-transparent",
        variant === "secondary" && "bg-secondary hover:bg-secondary-hover text-secondary-foreground disabled:bg-secondary/50 disabled:text-secondary-foreground/60 border-transparent",
        variant === "tertiary" && "bg-transparent hover:bg-surface-alt text-foreground border-transparent",
        variant === "danger" && "bg-secondary hover:bg-secondary-hover text-secondary-foreground border-primary",
        variant === "ghost" && "bg-transparent hover:bg-surface-alt text-foreground border-transparent",
        variant === "outline" && "bg-transparent border-border hover:bg-surface-alt text-foreground",
        options.class,
    );
};
