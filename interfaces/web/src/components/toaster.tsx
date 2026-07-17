import { Toast } from "@kobalte/core/toast";

export const Toaster = () => (
    <Toast.Region limit={10} swipeDirection="right">
        <Toast.List class="fixed bottom-0 right-0 flex flex-col gap-2 w-[400px] max-w-screen m-0 z-50 outline-none p-2" />
    </Toast.Region>
);
