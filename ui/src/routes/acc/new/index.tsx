import { createFileRoute, Link } from "@tanstack/solid-router";
import { FaSolidEye, FaSolidKey, FaSolidShield } from "solid-icons/fa";
import { For } from "solid-js";

const ACC_OPTIONS = [
  {
    label: "New Account",
    options: [
      {
        label: "Mnemonic",
        icon: FaSolidEye,
        href: "/acc/new/key",
      },
      {
        label: "Private Key",
        icon: FaSolidShield,
        href: "/acc/new/key",
      },
      {
        label: "Multisig",
        icon: FaSolidKey,
        disabled: true,
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
        label: "View",
        icon: FaSolidEye,
        href: "/acc/import/view",
      },
      {
        label: "Private Key",
        icon: FaSolidKey,
        href: "/acc/import/key",
      },
      {
        label: "Mnemonic",
        icon: FaSolidEye,
        href: "/acc/import/key",
      },
      {
        label: "Multisig",
        icon: FaSolidKey,
        disabled: true,
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
];

export const Route = createFileRoute("/acc/new/")({
  component: () => (
    <div class="w-full p-4">
      <div class="mx-auto w-full max-w-lg space-y-4 mt-4">
        <div class="flex items-end justify-between">
          <div class="text-xl">
            New Account
          </div>
        </div>
        <div class="bg-surface p-4 rounded-md w-full">
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
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
                                "w-full px-4 py-2 text-sm font-bold flex gap-2 items-center bg-surface-alt",
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
