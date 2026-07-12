import { createFileRoute, Link, useNavigate } from "@tanstack/solid-router";
import { FiPlus } from "solid-icons/fi";
import { createMemo, createSignal, Show, Suspense } from "solid-js";

import type { AccountLayout } from "#/api/account";
import { buildLayoutUpdate, useAccountLayout, useUpdateAccountLayout } from "#/api/account";
import { AccountsList, cloneLayout } from "#/components/account/list";
import { button } from "#/components/input/button";
import { Navbar } from "#/components/navbar";

export const Route = createFileRoute("/")({
  component: () => {
    const navigate = useNavigate();
    const layoutQuery = useAccountLayout();
    const [editing, setEditing] = createSignal(false);
    const [draftLayout, setDraftLayout] = createSignal<AccountLayout | undefined>(undefined);

    const updateLayout = useUpdateAccountLayout(({ data }) => ({
      contentType: "application/json; charset=utf-8",
      data,
    }));

    const layout = createMemo(() => layoutQuery.data);

    const beginEdit = () => {
      const current = layout();

      if (!current) return;

      setDraftLayout(cloneLayout(current));
      setEditing(true);
    };

    const finishEdit = () => {
      const draft = draftLayout();

      if (!draft) {
        setEditing(false);

        return;
      }

      updateLayout.mutate({ data: buildLayoutUpdate(draft) }, {
        onSuccess: () => {
          setEditing(false);
          setDraftLayout(undefined);
        },
      });
    };

    const cancelEdit = () => {
      setEditing(false);
      setDraftLayout(undefined);
    };

    const isEmpty = createMemo(() => (layout()?.accounts.length ?? 0) === 0);

    return (
      <>
        <Navbar />
        <div class="w-full p-4">
          <div class="mx-auto w-full max-w-2xl space-y-6">
            <div class="flex items-end justify-between gap-2">
              <div class="text-3xl font-bold">
                Accounts
              </div>
              <div class="flex items-center gap-2">
                <Show
                  when={editing()}
                  fallback={(
                    <button class={button({ variant: "outline", class: "text-sm" })} onClick={beginEdit}>
                      Edit
                    </button>
                  )}
                >
                  <button class={button({ variant: "ghost", class: "text-sm" })} onClick={cancelEdit}>
                    Cancel
                  </button>
                  <button class={button({ variant: "primary", class: "text-sm" })} onClick={finishEdit}>
                    Done
                  </button>
                </Show>
                <Link to="/acc/new" class={button({ variant: "primary", class: "text-sm" })}>
                  <FiPlus />
                  Add
                </Link>
              </div>
            </div>
            <div class="bg-surface py-5 px-4 rounded-xl w-full">
              <Suspense fallback={<div class="py-8 text-center text-muted">Loading...</div>}>
                <Show
                  when={!isEmpty()}
                  fallback={(
                    <button
                      type="button"
                      class="w-full px-2 hover:bg-surface-alt rounded-lg py-6 text-sm text-muted flex items-center justify-center gap-3"
                      onClick={() => navigate({ to: "/acc/new" })}
                    >
                      <div class="size-8 rounded-full border border-dashed border-border flex items-center justify-center">
                        <FiPlus class="size-4" />
                      </div>
                      Add your first account
                    </button>
                  )}
                >
                  <AccountsList
                    layout={layout}
                    editing={editing}
                    draftLayout={draftLayout}
                    onDraftChange={setDraftLayout}
                  />
                </Show>
              </Suspense>
            </div>
          </div>
        </div>
      </>
    );
  },
});
