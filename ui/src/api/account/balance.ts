import { createQuery } from "@tanstack/solid-query";

export const useAccountBalance = (account_identity: string) => createQuery(() =>
// a thing

({
    queryKey: ["account", account_identity, "balance"],
    queryFn: async () => {
        console.log("refetching balance");
        // const response = await api("/acc/{account_identity}/balance", "get", {
        //     path: {
        //         account_identity,
        //     },
        // });

        await new Promise(resolve => setTimeout(resolve, 3000));

        return {
            balance: 1_234_567_890n,
            asset: "fiat:usd",
            updated_at: new Date(),
        };
    },
    staleTime: 10_000,
}),
);
