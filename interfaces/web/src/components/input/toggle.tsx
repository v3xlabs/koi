import { Switch } from "@kobalte/core/switch";
import { Accessor, Component } from "solid-js";

export type ToggleProperties = {
    value: Accessor<boolean>;
    label?: string;
    description?: string;
    disabled?: boolean;
    onChange: (value: boolean) => void;
};

export const Toggle: Component<ToggleProperties> = ({ value = () => false, onChange = () => {}, label, description, disabled = false }) => (
    <Switch
      checked={value()}
      onChange={onChange}
      disabled={disabled}
      classList={{ "switch": true, "w-full flex justify-between items-center": !!(label || description), "opacity-50": disabled }}
    >
        {(label || description) && (
            <div>
                {label && (
                    <Switch.Label class="">
                        {label}
                    </Switch.Label>
                )}
                {description && (
                    <Switch.Description class="text-muted">
                        {description}
                    </Switch.Description>
                )}
            </div>
        )}
        <Switch.Input class="switch-input" />
        <Switch.Control class="switch-control">
            <Switch.Thumb class="switch-thumb" />
        </Switch.Control>
    </Switch>
);
