import { createFileRoute, Outlet, useCanGoBack, useMatches, useRouter } from "@tanstack/solid-router";
import { FiArrowLeft } from "solid-icons/fi";
import { createMemo, Show } from "solid-js";

import { button, cn } from "#/components/input/button";
import { Navbar } from "#/components/navbar";

export const Route = createFileRoute("/acc/_n")({
  component: () => {
    const router = useRouter();
    const canGoBack = useCanGoBack();
    const x = useMatches({});
    const metadata = createMemo(() => x().findLast(match => match.staticData));

    const className = createMemo(() => metadata()?.staticData?.className ?? "max-w-lg");

    return (
      <>
        <Navbar />
        <div class={cn("w-full mx-auto relative px-4", className())}>
          <div class="flex items-center gap-2">
            <Show when={canGoBack()}>
              <button class={cn(button({ variant: "secondary" }), "md:-ml-12")} onClick={() => router.history.back()}>
                <FiArrowLeft />
              </button>
            </Show>
            <Show when={metadata()?.staticData?.title}>
              {title => (
                <div class="text-xl font-bold">
                  {title()}
                </div>
              )}
            </Show>
          </div>
          <Outlet />
        </div>
      </>
    );
  },
});
