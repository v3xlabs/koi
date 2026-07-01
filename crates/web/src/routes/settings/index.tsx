import { createFileRoute } from "@tanstack/solid-router";
import { FiMonitor, FiMoon, FiSun } from "solid-icons/fi";
import { Component } from "solid-js";

import { useTheme } from "#/api/context";
import { SegmentedControl } from "#/components/input/segmented";
import { DisplayCurrencySelector } from "#/components/quoter/display";

export const Route = createFileRoute("/settings/")({
  component: () => (
    <div class="w-full space-y-4">
      <div class="">
        <div class="text-xl font-bold">
          Settings
        </div>
        <div class="text-sm text-muted">
          These are system-wide general settings.
        </div>
      </div>
      <div class="bg-surface rounded-md p-4 space-y-4">
        <DisplayCurrencySelector showLabel />
        <ThemeSelector />
      </div>
    </div>
  ),
});

const ThemeSelector: Component = () => {
  const { theme, setTheme } = useTheme();

  const options = [
    { value: "light" as const, label: "Light", icon: FiSun },
    { value: "dark" as const, label: "Dark", icon: FiMoon },
    { value: "system" as const, label: "System", icon: FiMonitor },
  ];

  return (
    <div>
      <label class="mb-1 block text-sm text-muted">Theme</label>
      <SegmentedControl
        value={theme()}
        onChange={setTheme}
        class="w-full"
      >
        <SegmentedControl.Control class="w-full">
          <SegmentedControl.Indicator />
          <div class="flex w-full relative">
            {options.map(opt => (
              <SegmentedControl.Item value={opt.value} class="flex-1">
                <SegmentedControl.ItemInput class="" />
                <SegmentedControl.ItemLabel class="flex items-center justify-center gap-1.5">
                  <opt.icon class="w-3.5 h-3.5" />
                  {opt.label}
                </SegmentedControl.ItemLabel>
              </SegmentedControl.Item>
            ))}
          </div>
        </SegmentedControl.Control>
      </SegmentedControl>
    </div>
  );
};
