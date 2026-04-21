import { createQuery } from "@tanstack/solid-query";

export const useAccountBalance = (account_id: string) => createQuery(() => {
    // a thing
    const x = "";

    return {
        queryKey: ["account", account_id, "balance"],
        queryFn: async () => {
            console.log("refetching balance");
            // const response = await api("/acc/{account_id}/balance", "get", {
            //     path: {
            //         account_id,
            //     },
            // });

            await new Promise(resolve => setTimeout(resolve, 3000));

            return {
                balance: 1_234_567_890n,
                asset: "fiat:usd",
                updated_at: new Date(),
            };
        },
    };
});
