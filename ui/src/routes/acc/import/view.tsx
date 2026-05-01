import { createFileRoute } from "@tanstack/solid-router";
import { createMemo, createSignal } from "solid-js";

import { useCreateAccount, useNextAccountId } from "#/api/account";
import { AddressInput } from "#/components/input/address";
import { NetworkSelect } from "#/components/net/input";

export const Route = createFileRoute("/acc/import/view")({
  component: () => {
    const nextAccountId = useNextAccountId();
    const [address, setAddress] = createSignal("");
    const [name, setName] = createSignal("");
    const [networks, setNetworks] = createSignal<number[]>([]);
    const createAccount = useCreateAccount(({ data: { account_identity, name, networks, address } }: { data: { account_identity: number; name: string; networks: number[]; address: string; }; }) => ({
      contentType: "application/json; charset=utf-8",
      data: { account_identity, name, networks, metadata: { type: "view", evm_address: address } },
    }));

    const handleClick = () => {
      const account_identity = nextAccountId.data;

      if (!account_identity || account_identity <= 0) return;

      if (networks().length === 0) return;

      createAccount.mutate({ data: { account_identity, name: name(), networks: networks(), address: address() } });
    };

    const disabled = createMemo(() => createAccount.isPending || !address() || !name() || networks().length === 0);

    return (
      <div class="p-4 mx-auto w-full max-w-lg">
        <div>
          Import View
        </div>
        <div class="bg-surface p-4 rounded-md w-full space-y-4">
          <div class="space-y-4">
            <label class="space-y-1 block">
              <span class="block">Name</span>
              <input type="text" class="input w-full" value={name()} onChange={e => setName(e.target.value)} />
            </label>
            <label class="space-y-1 block">
              <span class="block">Address</span>
              <AddressInput placeholder="0x123...456" class="w-full" value={address} onChange={setAddress} />
            </label>
            <label class="space-y-1 block">
              <span class="block">Networks</span>
              <NetworkSelect value={networks} onChange={setNetworks} />
              <p class="text-sm text-gray-500">
                {networks().length}
                {" "}
                networks selected
              </p>
            </label>
          </div>
          <div class="flex justify-end">
            <button class="btn btn-primary" onClick={handleClick} disabled={disabled()}>
              Import
            </button>
          </div>
        </div>
      </div>
    );
  },
});
