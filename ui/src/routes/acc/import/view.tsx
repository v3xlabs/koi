import { createFileRoute } from "@tanstack/solid-router";
import { createSignal } from "solid-js";

import { useCreateAccount, useNextAccountId } from "#/api/account";
import { AddressInput } from "#/components/input/address";

export const Route = createFileRoute("/acc/import/view")({
  component: () => {
    const nextAccountId = useNextAccountId();
    const [address, setAddress] = createSignal("");
    const [name, setName] = createSignal("");
    const createAccount = useCreateAccount(({ data: { account_id, name, address } }: { data: { account_id: number; name: string; address: string; }; }) => ({
      contentType: "application/json; charset=utf-8",
      data: { account_id, name, networks: [1], metadata: { type: "view", evm_address: address } },
    }));

    const handleClick = () => {
      const account_id = nextAccountId.data;

      if (!account_id || account_id <= 0) return;

      createAccount.mutate({ data: { account_id, name: name(), address: address() } });
    };

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
          </div>
          <div class="flex justify-end">
            <button class="btn btn-primary" onClick={handleClick} disabled={createAccount.isPending || !address() || !name()}>
              Import
            </button>
          </div>
        </div>
      </div>
    );
  },
});
