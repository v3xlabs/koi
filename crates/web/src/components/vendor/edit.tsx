import { Accessor, Component, createMemo, For, Show, Suspense } from "solid-js";

import { useAllVendors, useVendorFlagDisable, useVendorFlagEnable, useVendors, VendorFlagInfo } from "#/api/vendor";
import { capitalFirst } from "#/utils/text";

import { Toggle } from "../input/toggle";

export const VendorFlagToggle: Component<{ info: VendorFlagInfo; state: Accessor<boolean>; }> = ({ info, state }) => {
    const enabled = createMemo(() => state());
    const enableMutation = useVendorFlagEnable(() => ({ path: { flag: info.flag } }));
    const disableMutation = useVendorFlagDisable(() => ({ path: { flag: info.flag } }));

    return (
        <div class={info.unfinished ? "opacity-50 w-full" : "w-full"}>
            <Toggle
              value={() => enabled()}
              label={info.comment + (info.unfinished ? "*" : "")}
              description={info.flag}
              onChange={() => {
                    if (enabled()) {
                        disableMutation.mutate({});
                    }
                    else {
                        enableMutation.mutate({});
                    }
                }}
            />
        </div>
    );
};

export const VendorEdit: Component = () => {
    const vendorsQuery = useVendors();
    const allVendorsQuery = useAllVendors();

    const enabledVendors = createMemo(() => vendorsQuery.data?.vendors ?? []);

    // grouped by first word of the string
    const allVendorsGrouped = createMemo(() => Object.entries(allVendorsQuery.data?.vendors.reduce((acc, vendor) => {
        const firstWord = vendor.flag.toString().split("_")[0];

        if (!acc[firstWord]) {
            acc[firstWord] = [];
        }

        acc[firstWord].push(vendor);

        return acc;
    }, {} as Record<string, VendorFlagInfo[]>) ?? {}));

    return (
        <div>
            <div>
                Vendors
            </div>
            <Suspense fallback={<div>Loading...</div>}>
                <Show when={allVendorsGrouped()}>
                    {data => (
                        <div class="space-y-2">
                            <For each={data()}>
                                {group => (
                                    <div class="bg-surface p-4 rounded-md space-y-2">
                                        <div class="flex justify-between items-center">
                                            <div class="font-bold">
                                                {capitalFirst(group[0])}
                                            </div>
                                            <label class="group flex justify-between items-center gap-2">
                                                <span class="opacity-0 group-hover:opacity-100 transition-opacity duration-200">
                                                    Enable all
                                                </span>
                                                <Toggle
                                                  value={() => group[1].every(vendor => enabledVendors().includes(vendor.flag))}
                                                  onChange={() => {
                                                        const direction = group[1].every(vendor => enabledVendors().includes(vendor.flag)) ? "enable" : "disable";

                                                        // TODO: Implement
                                                        // if (direction === "enable") {
                                                        //     group[1].forEach(vendor => enableVendorFlag(vendor.flag));
                                                        // }
                                                        // else {
                                                        //     group[1].forEach(vendor => disableVendorFlag(vendor.flag));
                                                        // }
                                                    }}
                                                />
                                            </label>
                                        </div>
                                        <ul class="space-y-2">
                                            <For each={group[1]}>
                                                {vendor => (
                                                    <li class="flex justify-between items-center">
                                                        <VendorFlagToggle
                                                          info={vendor}
                                                          state={() => enabledVendors().includes(vendor.flag)}
                                                        />
                                                    </li>
                                                )}
                                            </For>
                                        </ul>
                                    </div>
                                )}
                            </For>
                        </div>
                    )}
                </Show>
            </Suspense>
        </div>
    );
};
