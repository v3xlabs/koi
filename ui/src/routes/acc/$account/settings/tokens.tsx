import { createFileRoute, useParams } from "@tanstack/solid-router";

import { useAssets } from "#/api/asset";

export const Route = createFileRoute("/acc/$account/settings/tokens")({
  component: () => {
    const params = useParams({ from: "/acc/$account" });

    const assetsQuery = useAssets(() => ({}));

    return (
      <div class="px-4">
        <div class="bg-surface p-4 rounded-md w-full space-y-4">
          <div>
            {JSON.stringify(assetsQuery.data)}
          </div>
          <div class="flex justify-end">
            <button class="btn btn-primary">
              Save
            </button>
          </div>
        </div>
      </div>
    );
  },
});
