import { createFileRoute } from "@tanstack/solid-router";

export const Route = createFileRoute("/acc/new/key")({
  component: () => (
    <div class="p-4 mx-auto w-full max-w-lg">
      <div class="bg-surface p-4 rounded-md w-full">
        <div>
          <div>
            New Key
          </div>
        </div>
      </div>
    </div>
  ),
});
