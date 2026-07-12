import { Tooltip } from "@kobalte/core/tooltip";
import { formatDistanceToNow } from "date-fns";
import { Component, createSignal, onCleanup, Show } from "solid-js";

type TimeValue = string | number | Date | null | undefined;

const SECOND = 1000;
const MONTH = 30 * 24 * 60 * 60 * SECOND;

const toDate = (v: TimeValue) => (v == null || v === "" ? null : new Date(v));

export const FormattedTime: Component<{ value: TimeValue; class?: string; prefix?: string; }> = (props) => {
  const [now, setNow] = createSignal(Date.now());
  const interval = setInterval(() => setNow(Date.now()), SECOND);

  onCleanup(() => clearInterval(interval));

  return (
    <Show when={toDate(props.value)} fallback={<time class={props.class}>-</time>}>
      {(date) => {
        const absolute = () => date().toLocaleString();
        const isRelative = () => Math.abs(date().getTime() - now()) <= MONTH;
        const display = () => {
          if (!isRelative()) return absolute();

          return formatDistanceToNow(date(), { addSuffix: true });
        };

        return (
          <Tooltip placement="top">
            <Tooltip.Trigger
              as="time"
              class={props.class}
              dateTime={date().toISOString()}
              tabIndex={0}
            >
              {props.prefix && (
                <span>
                  {props.prefix}
                </span>
              )}
              {display()}
            </Tooltip.Trigger>
            <Tooltip.Portal>
              <Tooltip.Content class="bg-surface-alt text-secondary-foreground text-sm p-2 rounded-md border border-border">
                <Tooltip.Arrow />
                {isRelative() && <p>{absolute()}</p>}
                <p>
                  Unix:
                  {Math.floor(date().getTime() / SECOND)}
                </p>
              </Tooltip.Content>
            </Tooltip.Portal>
          </Tooltip>
        );
      }}
    </Show>
  );
};
