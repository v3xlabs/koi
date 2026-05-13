import { Select } from "@kobalte/core/select";
import { FiCheck, FiChevronDown } from "solid-icons/fi";
import { Component, createMemo, Show, useContext } from "solid-js";

import { appcontext } from "#/api";
import { Asset, useAssets } from "#/api/asset";

export const DisplayCurrencySelector: Component<{ showLabel?: boolean; }> = (props) => {
    const assets = useAssets();
    const fiatAssets = createMemo(() => assets.data?.assets?.filter(asset => asset.asset_identity.startsWith("fiat:")) ?? []);
    const { displayCurrency: [selected, setSelected] } = useContext(appcontext);

    return (
        <div>
            <Select<Asset>
              multiple={false}
              optionValue="asset_identity"
              optionTextValue="asset_name"
              value={fiatAssets().find(asset => asset.asset_identity === selected())}
              options={fiatAssets()}
              itemComponent={props => (
                    <Select.Item item={props.item} class="select__item">
                        <Select.ItemLabel>{props.item.rawValue.asset_identity}</Select.ItemLabel>
                        <Select.ItemIndicator class="select__item-indicator">
                            <FiCheck />
                        </Select.ItemIndicator>
                    </Select.Item>
                )}
              onChange={x => x?.asset_identity && setSelected(x?.asset_identity)}
            >
                <Show when={props.showLabel}>
                    <Select.Label class="mb-1 block text-sm text-muted">Display Currency</Select.Label>
                </Show>
                <Select.Trigger class="select__trigger min-w-32">
                    <Select.Value<Asset>>
                        {state => state.selectedOption()?.asset_identity}
                    </Select.Value>
                    <Select.Icon class="text-muted">
                        <FiChevronDown />
                    </Select.Icon>
                </Select.Trigger>
                <Select.Portal>
                    <Select.Content class="select__content min-w-32">
                        <Select.Arrow />
                        <Select.Listbox />
                    </Select.Content>
                </Select.Portal>
            </Select>
        </div>
    );
};
