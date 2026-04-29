import { Tooltip } from "@kobalte/core/tooltip";
import { FaSolidEye, FaSolidQuestion, FaSolidUser } from "solid-icons/fa";
import { Accessor, Component, createMemo } from "solid-js";
import { match } from "ts-pattern";

import { WalletType } from "#/api/account";
import SafeIcon from "#/assets/safe-icon.svg";

export const AccountTypeIcon: Component<{ type: Accessor<WalletType["type"]>; }> = (props) => {
    const icon = createMemo(() => match(props.type())
        .with("eoa", () => FaSolidUser)
        .with("view", () => FaSolidEye)
        .with("safe", () => () => <img src={SafeIcon} alt="Safe Multisig" class="size-3.5" />)
        .otherwise(() => FaSolidQuestion));

    const tooltip = createMemo(() => match(props.type())
        .with("eoa", () => "Externally Owned Account")
        .with("view", () => "View-only")
        .with("safe", () => "Safe Multisig")
        .with("railgun", () => "Railgun")
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
