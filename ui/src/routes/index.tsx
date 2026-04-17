import { createFileRoute, Link } from "@tanstack/solid-router";
import { For } from "solid-js";

export const Route = createFileRoute("/")({
  component: () => {
    // something
    Route.useParams();

    return (
      <div class="w-full p-4">
        <div class="mx-auto w-full max-w-md">
          <div>
            hi
          </div>
          <div class="bg-surface py-4 rounded-md w-full">
            <div class="space-y-2">
              <For each={[1, 2, 3]}>
                {item => (
                  <Link to="/acc/$account" params={{ account: item.toString() }} class="w-full px-2 py-1 bg-surface-alt text-sm font-bold flex">
                    {item}
                  </Link>
                )}
              </For>
            </div>
          </div>
        </div>
      </div>
    );
  },
});
