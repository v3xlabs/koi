import { Toast, toaster } from "@kobalte/core/toast";
import { createFileRoute, useParams } from "@tanstack/solid-router";
import { Accessor, Component, createMemo, createSignal, Show, Suspense } from "solid-js";

import { Account, useAccount, useUpdateAccount } from "#/api/account";
import { NetworkSelect } from "#/components/net/input";

export const Route = createFileRoute("/acc/$account/settings/")({
  component: () => {
    const params = useParams({ from: "/acc/$account" });
    const accountQuery = useAccount(() => ({ path: { account_id: Number.parseInt(params().account) } }));
    const account = createMemo(() => accountQuery.data);

    return (
      <div class="px-4">
        <Suspense>
          <Show when={account()}>
            {data => (
              <AccountEdit account={data} />
            )}
          </Show>
        </Suspense>
      </div>
    );
  },
});

const AccountEdit: Component<{ account: Accessor<Account>; }> = ({ account }) => {
  const [chosenName, setName] = createSignal(account()?.name ?? "");
  const [selectedNetworks, setNetworks] = createSignal<number[]>(account()?.networks ?? []);

  const isDirty = createMemo(() => chosenName() !== account()?.name || selectedNetworks() !== account()?.networks);

  const updateAccount = useUpdateAccount(({ data }: { data: Account; }) => ({
    path: { account_id: account()?.account_id },
    contentType: "application/json; charset=utf-8",
    data,
  }));

  const mutate = () => {
    // TODO:
    const account_id = account()?.account_id;
    const metadata = account()?.metadata;
    const name = chosenName();
    const networks = selectedNetworks();

    updateAccount.mutate({ data: { account_id, metadata, name, networks } }, {
      onSuccess: () => {
        toaster.show(props => (
          <Toast toastId={props.toastId} class="toast">
            <div class="flex justify-between items-center">
              <div>Account updated</div>
            </div>
          </Toast>
        ));
      },
    });
  };

  return (
    <div class="bg-surface p-4 rounded-md w-full space-y-4">
      <div>
        <div>Name</div>
        <input type="text" class="input w-full" value={chosenName()} onChange={e => setName(e.target.value)} />
      </div>
      <div>
        <div>Networks</div>
        <NetworkSelect value={selectedNetworks} onChange={setNetworks} />
      </div>
      <div class="flex justify-end">
        <button class="btn btn-primary" disabled={!isDirty()} onClick={mutate}>
          Save
        </button>
      </div>
    </div>
  );
};
