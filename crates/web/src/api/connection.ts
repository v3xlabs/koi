import { queryClient } from "./client";
import { createApi, createApiMutation } from "./query";
import { components } from "./schema.gen";

export type Connection = components["schemas"]["Connection"];

export const connectionKeys = {
    all: ["connections"] as const,
};

const invalidateConnections = () => {
    queryClient.invalidateQueries({ queryKey: connectionKeys.all });
};

export const useConnections = createApi("/connections", "get", () => connectionKeys.all);

export const useAddConnection = createApiMutation("/connections", "post", {
    onSuccess: invalidateConnections,
});

export const useDisconnectConnection = createApiMutation(
    "/connections/{connection_id}/disconnect",
    "post",
    {
        onSuccess: invalidateConnections,
    },
);

export const useRemoveConnection = createApiMutation("/connections/{connection_id}", "delete", {
    onSuccess: invalidateConnections,
});
