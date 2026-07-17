import { createFileRoute, Link } from "@tanstack/solid-router";
import { FaSolidEye, FaSolidKey, FaSolidShield } from "solid-icons/fa";
import { For } from "solid-js";

import SafeIcon from "#/assets/safe-icon.svg";

const ACC_OPTIONS = [
  {
    label: "New Account",
    options: [
      {
        label: "Mnemonic",
        icon: FaSolidEye,
        href: "/acc/new/mnemonic",
      },
      {
        label: "Private Key",
        icon: FaSolidShield,
        href: "/acc/new/private-key",
      },
      {
        label: "Multisig",
        icon: () => <img src={SafeIcon} alt="Safe Multisig" class="size-3.5" />,
        href: "/acc/new/safe",
      },
      {
        label: "Frost",
        icon: FaSolidKey,
        disabled: true,
      },
    ],
  },
  {
    label: "Import",
    options: [
      {
        label: "Multisig",
        icon: () => <img src={SafeIcon} alt="Safe Multisig" class="size-3.5" />,
        href: "/acc/import/safe",
      },
      {
        label: "View",
        icon: FaSolidEye,
        href: "/acc/import/view",
      },
      {
        label: "Mnemonic",
        icon: FaSolidEye,
        href: "/acc/import/mnemonic",
      },
      {
        label: "Private Key",
        icon: FaSolidKey,
        href: "/acc/import/private-key",
      },
      {
        label: "Frost",
        icon: FaSolidKey,
        disabled: true,
      },
      {
        label: "Hardware",
        icon: FaSolidKey,
        disabled: true,
      },
    ],
  },
  {
    label: "Hardware",
    options: [
      {
        label: "Ledger",
        icon: FaSolidEye,
        disabled: true,
      },
      {
        label: "Trezor",
        icon: FaSolidKey,
        disabled: true,
      },
    ],
  },
];

export const Route = createFileRoute("/acc/_n/new/")({
  staticData: {
    title: "New Account",
    className: "max-w-3xl",
  },
  component: () => (
    <div class="w-full">
      <div class="mx-auto w-full max-w-3xl space-y-4">
        <div class="bg-surface p-4 rounded-md w-full">
          <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <For each={ACC_OPTIONS}>
              {group => (
                <div class="space-y-2">
                  <div class="border-b border-border pb-2">
                    {group.label}
                  </div>
                  <ul class="space-y-1">
                    <For each={group.options}>
                      {option => (
                        <li class="w-full">
                          <Link
                            to={option.href}
                            disabled={option.disabled || !option.href}
                            class={
                              [
                                "w-full px-4 py-2 text-sm font-bold flex gap-2 items-center bg-surface-alt rounded-md",
                                option.disabled && "cursor-not-allowed opacity-50",
                                !option.disabled && "hover:bg-surface",
                              ].filter(Boolean).join(" ")
                            }
                          >
                            <option.icon class="w-3.5 h-3.5" />
                            {option.label}
                          </Link>
                        </li>
                      )}
                    </For>
                  </ul>
                </div>
              )}
            </For>
          </div>
        </div>
      </div>
    </div>
  ),
});
