import { createFileRoute, useNavigate } from "@tanstack/solid-router";
import { createMemo, createSignal } from "solid-js";

import { useCreateAccount } from "#/api/account";
import { AddressInput } from "#/components/input/address";
import { button } from "#/components/input/button";
import { NetworkSelect } from "#/components/net/input";

export const Route = createFileRoute("/acc/_n/import/view")({
  staticData: {
    title: "Import View",
  },
  component: () => {
    const navigate = useNavigate();
    const [address, setAddress] = createSignal("");
    const [name, setName] = createSignal("");
    const [networks, setNetworks] = createSignal<number[]>([]);
    const createAccount = useCreateAccount(({ data: { name, networks, address, display_order } }: { data: { name: string; networks: number[]; address: string; display_order: number; }; }) => ({
      contentType: "application/json; charset=utf-8",
      data: { name, networks, display_order, metadata: { type: "view", evm_address: address } },
    }));

    const handleClick = async () => {
      if (networks().length === 0) return;

      const account = await createAccount.mutateAsync({ data: { name: name(), networks: networks(), display_order: 0, address: address() } });

      navigate({ to: "/acc/$account", params: { account: account.account_identity.toString() } });
    };

    const disabled = createMemo(() => createAccount.isPending || !address() || !name() || networks().length === 0);

    return (
      <div class="bg-surface p-4 rounded-md w-full space-y-4">
        <div class="space-y-4">
          <label class="space-y-1 block">
            <span class="block">Name</span>
            <input
              type="text"
              class="input w-full"
              value={name()}
              onChange={e => setName(e.target.value)}
            />
          </label>
          <label class="space-y-1 block">
            <span class="block">Address</span>
            <AddressInput
              placeholder="0x123...456"
              class="w-full"
              value={address}
              onChange={setAddress}
            />
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
          <button class={button({ variant: "primary" })} onClick={handleClick} disabled={disabled()}>
            Import
          </button>
        </div>
      </div>
    );
  },
});
