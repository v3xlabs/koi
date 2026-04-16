import { Modal } from "#/components/dialog";
import { Accessor, createMemo, createSignal, For, ParentComponent } from "solid-js";
import { encodeQR } from 'qr';
import { match } from "ts-pattern";

type ReceiveQRProperties = {
    address: Accessor<string>;
}

const supportedQRType = [
    'raw',
    'safe',
    'erc681',
] as const;

type QRType = typeof supportedQRType[number];

export const ReceiveQR: ParentComponent<ReceiveQRProperties> = (props) => {
    const [qrType, setQRType] = createSignal<QRType>('raw');
    const url = () => match({ type: qrType() })
        .with({ type: 'raw' }, () => props.address())
        // TODO: Add support for non-mainnet chains
        .with({ type: 'safe' }, () => `eth:${props.address()}`)
        .with({ type: 'erc681' }, () => `ethereum:${props.address()}`)
        .exhaustive();
    const qr = createMemo(() => encodeQR(url(), 'svg'));
    const qrImage = createMemo(() => `data:image/svg+xml;base64,${btoa(qr())}`);

    return (
        <Modal>
            {props.children}
            <Modal.Portal>
                <Modal.Overlay />
                <div class="fixed inset-0">
                    <Modal.Content class="w-full max-w-xl bg-surface rounded-md relative mx-auto mt-24">
                        <Modal.CloseButton />
                        <Modal.Title>
                            Wallet Address QR
                        </Modal.Title>

                        <div class="p-4">
                            <div class="flex flex-col md:flex-row gap-4">
                                <div class="w-48 h-48 border border-border bg-[#ffff] rounded-md">
                                    <img src={qrImage()} alt="QR Code" class="w-full h-full object-contain" />
                                </div>
                                <div>
                                    <div class="flex gap-2">
                                        <For each={supportedQRType}>
                                            {(type) => (
                                                <button
                                                    classList={{
                                                        "bg-surface hover:bg-surface-alt rounded-md px-2 py-1 text-sm font-bold flex items-center gap-2 cursor-pointer": true,
                                                        "bg-primary hover:bg-primary-hover text-primary-foreground": type === qrType(),
                                                    }}
                                                    onClick={() => setQRType(type)}
                                                >
                                                    {type}
                                                </button>
                                            )}
                                        </For>
                                    </div>
                                    <div>
                                        <input type="text" class="w-full rounded-md px-2 py-1 text-sm font-bold flex items-center gap-2 cursor-pointer" value={url()} />
                                    </div>
                                </div>
                            </div>
                        </div>
                    </Modal.Content>
                </div>
            </Modal.Portal>
        </Modal>
    )
}
