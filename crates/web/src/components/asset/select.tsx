import { Combobox } from "@kobalte/core/combobox";
import { FiCheck, FiChevronDown } from "solid-icons/fi";
import { Accessor, Component, createMemo, Show } from "solid-js";

import { Asset, useAssets } from "#/api/asset";

import { AssetIcon } from "./icon";
import { AssetPreview } from "./preview";

type AssetSelectProps = {
    label: string;
    value: Accessor<string>;
    onChange: (value: string) => void;
    networkIdentity?: number;
};

type AssetGroup = {
    label: string;
    sort: number;
    options: Asset[];
};

const assetGroup = (asset: Asset): Omit<AssetGroup, "options"> => {
    const [kind, chainId] = asset.asset_identity.split(":");

    if (kind === "fiat") {
        return { label: "Fiat", sort: -1 };
    }

    if ((kind === "native" || kind === "erc20") && chainId) {
        return { label: `EVM Chain #${chainId}`, sort: Number(chainId) };
    }

    return { label: "Other", sort: Number.MAX_SAFE_INTEGER };
};

export const AssetSelect: Component<AssetSelectProps> = (props) => {
    const assetsQuery = useAssets();
    const assets = createMemo(() => {
        const all = assetsQuery.data?.assets ?? [];

        if (props.networkIdentity === undefined) return all;

        const networkId = props.networkIdentity.toString();

        return all.filter((asset) => {
            const [kind, chainId] = asset.asset_identity.split(":");

            if (kind === "fiat") return false;

            return chainId === networkId;
        });
    });
    const assetGroups = createMemo(() => {
        const groups = new Map<string, AssetGroup>();

        for (const asset of assets()) {
            const group = assetGroup(asset);
            const existing = groups.get(group.label);

            if (existing) {
                existing.options.push(asset);
            }
            else {
                groups.set(group.label, { ...group, options: [asset] });
            }
        }

        return [...groups.values()]
            .map(group => ({
                ...group,
                options: group.options.toSorted((a, b) => a.asset_name.localeCompare(b.asset_name)),
            }))
            .toSorted((a, b) => a.sort - b.sort || a.label.localeCompare(b.label));
    });
    const selectedAsset = createMemo(() => assets().find(asset => asset.asset_identity === props.value()));

    return (
        <Combobox<Asset, AssetGroup>
          optionValue="asset_identity"
          optionTextValue="asset_name"
          optionLabel="asset_name"
          optionGroupChildren="options"
          value={selectedAsset()}
          options={assetGroups()}
          defaultFilter={(asset, inputValue) => {
              const query = inputValue.trim().toLowerCase();

              return (
                  asset.asset_name.toLowerCase().includes(query)
                  || asset.asset_symbol.toLowerCase().includes(query)
                  || asset.asset_identity.toLowerCase().includes(query)
              );
          }}
          placeholder="Search assets..."
          triggerMode="focus"
          itemComponent={itemProps => (
                <Combobox.Item item={itemProps.item} class="select__item">
                    <Combobox.ItemLabel>
                        <AssetPreview asset_identity={itemProps.item.rawValue.asset_identity} />
                    </Combobox.ItemLabel>
                    <Combobox.ItemIndicator class="select__item-indicator">
                        <FiCheck />
                    </Combobox.ItemIndicator>
                </Combobox.Item>
            )}
          sectionComponent={sectionProps => (
                <Combobox.Section class="px-2 py-1 text-xs font-medium uppercase tracking-wide text-muted">
                    {sectionProps.section.rawValue.label}
                </Combobox.Section>
            )}
          onChange={asset => props.onChange(asset?.asset_identity ?? "")}
          onInputChange={(value) => {
                if (value === "") props.onChange("");
            }}
        >
            <Combobox.Label class="mb-1 block text-sm text-muted">{props.label}</Combobox.Label>
            <Combobox.Control aria-label={props.label} class="select__trigger w-full min-h-9">
                <div>
                    <Show when={props.value()}>
                        {asset_identity => <AssetIcon asset_identity={asset_identity()} />}
                    </Show>
                </div>
                <Combobox.Input class="min-w-0 flex-1 bg-transparent outline-none placeholder:text-muted" />
                <Combobox.Trigger class="text-muted">
                    <Combobox.Icon>
                        <FiChevronDown />
                    </Combobox.Icon>
                </Combobox.Trigger>
            </Combobox.Control>
            <Combobox.Portal>
                <Combobox.Content class="select__content w-full max-w-md max-h-80 overflow-y-auto">
                    <Combobox.Listbox />
                </Combobox.Content>
            </Combobox.Portal>
        </Combobox>
    );
};
