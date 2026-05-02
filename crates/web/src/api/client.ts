import { experimental_createQueryPersister } from "@tanstack/query-persist-client-core";
import { QueryClient } from "@tanstack/solid-query";

const persister = experimental_createQueryPersister({
  storage: localStorage,
  maxAge: 1000 * 60 * 60 * 12, // 12 hours
  prefix: "koi",
});

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30_000,
      persister: persister.persisterFn,
    },
  },
});
