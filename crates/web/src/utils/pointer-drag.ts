import { Accessor, createSignal, onCleanup } from "solid-js";

export type DragPoint = {
    x: number;
    y: number;
};

type PointerDragOptions<T> = {
    cursor?: string;
    onMove?: (item: T, point: DragPoint, event: PointerEvent) => void;
    onDrop?: (item: T, point: DragPoint, event: PointerEvent) => void;
    onStop?: () => void;
};

type PointerDrag<T> = {
    dragItem: Accessor<T | null>;
    pointer: Accessor<DragPoint | null>;
    startDrag: (item: T) => (event: PointerEvent) => void;
    stopDrag: () => void;
};

const pointFromEvent = (event: PointerEvent): DragPoint => ({
    x: event.clientX,
    y: event.clientY,
});

export const createPointerDrag = <T>(options: PointerDragOptions<T>): PointerDrag<T> => {
    const [dragItem, setDragItem] = createSignal<T | null>(null);
    const [pointer, setPointer] = createSignal<DragPoint | null>(null);

    const stopDrag = () => {
        setDragItem(null);
        setPointer(null);
        document.body.style.removeProperty("user-select");
        document.body.style.removeProperty("cursor");
        options.onStop?.();
    };

    onCleanup(stopDrag);

    const startDrag = (item: T) => (event: PointerEvent) => {
        event.preventDefault();
        event.stopPropagation();

        const handle = event.currentTarget as HTMLElement;

        handle.setPointerCapture(event.pointerId);
        setDragItem(() => item);
        setPointer(pointFromEvent(event));
        document.body.style.userSelect = "none";
        document.body.style.cursor = options.cursor ?? "grabbing";

        const onMove = (moveEvent: PointerEvent) => {
            const point = pointFromEvent(moveEvent);

            setPointer(point);
            options.onMove?.(item, point, moveEvent);
        };

        const onEnd = (endEvent: PointerEvent) => {
            if (handle.hasPointerCapture(endEvent.pointerId)) {
                handle.releasePointerCapture(endEvent.pointerId);
            }

            globalThis.removeEventListener("pointermove", onMove);
            globalThis.removeEventListener("pointerup", onEnd);
            globalThis.removeEventListener("pointercancel", onEnd);

            if (endEvent.type !== "pointercancel") {
                options.onDrop?.(item, pointFromEvent(endEvent), endEvent);
            }

            stopDrag();
        };

        globalThis.addEventListener("pointermove", onMove);
        globalThis.addEventListener("pointerup", onEnd);
        globalThis.addEventListener("pointercancel", onEnd);
    };

    return {
        dragItem,
        pointer,
        startDrag,
        stopDrag,
    };
};
