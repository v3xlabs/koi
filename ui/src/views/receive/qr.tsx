import { SegmentedControl } from "@kobalte/core/segmented-control";
import { encodeQR } from "qr";
import { Accessor, createMemo, createSignal, For, ParentComponent } from "solid-js";
import { match } from "ts-pattern";

import { Modal } from "#/components/dialog";

type ReceiveQRProperties = {
    address: Accessor<string>;
};

const supportedQRType = [
    "raw",
    "safe",
    "erc681",
] as const;

type QRType = typeof supportedQRType[number];

export const ReceiveQR: ParentComponent<ReceiveQRProperties> = (props) => {
    const [qrType, setQRType] = createSignal<QRType>("raw");
    const url = () => match({ type: qrType() })
        .with({ type: "raw" }, () => props.address())
        // TODO: Add support for non-mainnet chains
        .with({ type: "safe" }, () => `eth:${props.address()}`)
        .with({ type: "erc681" }, () => `ethereum:${props.address()}`)
        .exhaustive();
    const qr = createMemo(() => encodeQR(url(), "svg"));
    const qrImage = createMemo(() => `data:image/svg+xml;base64,${btoa(qr())}`);

    return (
        <Modal>
            {props.children}
            <Modal.Portal>
                <Modal.Overlay />
                <div class="fixed inset-0 z-50">
                    <Modal.Content class="w-full max-w-xl bg-surface rounded-md relative mx-auto mt-24 z-50">
                        <Modal.CloseButton />
                        <Modal.Title>
                            Wallet Address QR
                        </Modal.Title>

                        <div class="p-4">
                            <div class="flex flex-col md:flex-row gap-4">
                                <div class="w-48 h-48 border border-border bg-[#ffff] rounded-md">
                                    <img src={qrImage()} alt="QR Code" class="w-full h-full object-contain" />
                                </div>
                                <div class="space-y-2">
                                    <SegmentedControl
                                      value={qrType()}
                                      onChange={setQRType}
                                      class=""
                                    >
                                        <SegmentedControl.Label class="">
                                            Url Format
                                        </SegmentedControl.Label>
                                        <div class="relative border border-border rounded-md p-1 w-fit" role="presentation">
                                            <SegmentedControl.Indicator class="absolute top-1 left-1 w-full h-full bg-primary rounded-md transition-all duration-300" />
                                            <div class="flex gap-2 w-fit relative">
                                                <For each={supportedQRType}>
                                                    {type => (
                                                        <SegmentedControl.Item
                                                          value={type}
                                                          class="px-2"
                                                        >
                                                            <SegmentedControl.ItemInput class="" />
                                                            <SegmentedControl.ItemLabel class="cursor-pointer">
                                                                {type}
                                                            </SegmentedControl.ItemLabel>
                                                        </SegmentedControl.Item>
                                                    )}
                                                </For>
                                            </div>
                                        </div>
                                    </SegmentedControl>
                                    <div>
                                        <input
                                          type="text"
                                          class="w-full rounded-md px-2 py-1 font-bold flex items-center gap-2 cursor-pointer border border-border"
                                          value={url()}
                                          readonly
                                        />
                                    </div>
                                </div>
                            </div>
                        </div>
                    </Modal.Content>
                </div>
            </Modal.Portal>
        </Modal>
    );
};
