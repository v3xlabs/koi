import { Dialog, Root as DialogRoot } from "@kobalte/core/dialog";
import { FaSolidXmark } from "solid-icons/fa";
import { Component, JSX } from "solid-js";

const CloseButton: typeof Dialog.CloseButton = props => (props.children
  ? (
    <Dialog.CloseButton {...props}>{props.children}</Dialog.CloseButton>
  )
  : (
    <Dialog.CloseButton
      classList={{
        [props.class]: true,
        "absolute top-2 right-2 bg-surface rounded-md p-2 cursor-pointer hover:bg-surface-alt transition-colors": true,
      }}
      {...props}
    >
      <FaSolidXmark class="w-4 h-4" />
    </Dialog.CloseButton>
  ));

const Root: typeof DialogRoot = props => (
  <DialogRoot
    {...props}
  />
);

const Overlay: typeof Dialog.Overlay = props => (
  <Dialog.Overlay
    classList={{
      [props.class]: true,
      "fixed inset-0 z-50 bg-background/50": true,
    }}
    {...props}
  />
);

const Positioner: Component<{ class?: string; children: JSX.Element; }> = props => (
  <div
    classList={{
      "fixed inset-0 z-50": true,
      [props.class ?? ""]: true,
    }}
  >
    {props.children}
  </div>
);

const Title: typeof Dialog.Title = props => (
  <div
    classList={{
      [props.class]: true,
      "w-full border-b border-border pb-4 px-4 pt-4": true,
    }}
  >
    <Dialog.Title
      {...props}
    />
  </div>
);

const Content: typeof Dialog.Content = props => (
  <Dialog.Content
    classList={{
      "z-50": true,
      [props.class]: true,
    }}
    {...props}
  />
);

export const Modal = Object.assign(Root, {
  ...Dialog,
  CloseButton,
  Content,
  Overlay,
  Positioner,
  Title,
});
