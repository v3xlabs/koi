import { Dialog, Root as DialogRoot } from '@kobalte/core/dialog';
import { FaSolidXmark } from 'solid-icons/fa';

const CloseButton: typeof Dialog.CloseButton = (props) => {
    return (
        <Dialog.CloseButton
            classList={{
                [props.class]: true,
                "absolute top-2 right-2 bg-surface rounded-md p-2 cursor-pointer hover:bg-surface-alt transition-colors": true,
            }}
            {...props}
        >
            <FaSolidXmark class="w-4 h-4" />
        </Dialog.CloseButton>
    )
}

const Root: typeof DialogRoot = (props) => {
    return (
        <DialogRoot
            {...props}
        />
    )
}

const Overlay: typeof Dialog.Overlay = (props) => {
    return (
        <Dialog.Overlay
            classList={{
                [props.class]: true,
                "fixed inset-0 bg-background/50": true,
            }}
            {...props}
        />
    )
}

const Title: typeof Dialog.Title = (props) => {
    return (
        <Dialog.Title
            classList={{
                [props.class]: true,
                "w-full border-b border-border pb-4 px-4 pt-4": true,
            }}
            {...props}
        />
    )
}

export const Modal = Object.assign(Root, {
    ...Dialog,
    CloseButton,
    Overlay,
    Title,
});
