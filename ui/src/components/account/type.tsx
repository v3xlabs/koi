import { Tooltip } from "@kobalte/core/tooltip";
import { FaSolidEye, FaSolidQuestion, FaSolidUser } from "solid-icons/fa";
import { Accessor, Component, createMemo } from "solid-js";
import { match } from "ts-pattern";

import { WalletType } from "#/api/account";

export const AccountTypeIcon: Component<{ type: Accessor<WalletType["type"]>; }> = (props) => {
    const icon = createMemo(() => match(props.type())
        .with("EOA", () => FaSolidUser)
        .with("View", () => FaSolidEye)
        .otherwise(() => FaSolidQuestion));

    const tooltip = createMemo(() => match(props.type())
        .with("EOA", () => "Externally Owned Account")
        .with("View", () => "View-only")
        .with("Safe", () => "Safe Multisig")
        .with("Railgun", () => "Railgun")
        .otherwise(() => "Unknown"));

    return (
        <Tooltip placement="right">
            <Tooltip.Trigger>
                <div class="flex items-center justify-center pb-0.5">
                    {icon()({ class: "size-3" })}
                </div>
            </Tooltip.Trigger>
            <Tooltip.Portal>
                <Tooltip.Content class="bg-surface-alt text-secondary-foreground text-sm p-2 rounded-md border border-border">
                    <Tooltip.Arrow />
                    <p>
                        {tooltip()}
                    </p>
                </Tooltip.Content>
            </Tooltip.Portal>
        </Tooltip>
    );
};
