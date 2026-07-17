import { Toast, toaster } from "@kobalte/core/toast";
import { Command } from "cmdk-solid";
import { FaSolidRefresh } from "solid-icons/fa";
import { FiActivity } from "solid-icons/fi";
import { Show } from "solid-js";

import { api } from "#/api";
import { refreshAccountBalances, useAccountLayout } from "#/api/account";
import { useDisplayCurrency } from "#/api/context";

import { CommandGroupProperties, CommandMenuItem } from "./item";

const showToast = (message: string) => toaster.show(props => (
    <Toast toastId={props.toastId} class="toast">
        <div>{message}</div>
    </Toast>
));

export const DiagnosticCommands = (props: CommandGroupProperties) => {
    const layoutQuery = useAccountLayout();
    const { displayCurrency } = useDisplayCurrency();

    const refreshAllBalances = () => {
        const accounts = layoutQuery.data?.accounts ?? [];

        props.close();
        void Promise.allSettled(accounts.map(account => refreshAccountBalances({
            path: { account_identity: account.account_identity },
            query: { display_currency: displayCurrency() },
        }))).then((results) => {
            const failed = results.filter(result => result.status === "rejected").length;
            const refreshed = results.length - failed;

            showToast(failed === 0
                ? `Refreshed ${refreshed} account${refreshed === 1 ? "" : "s"}`
                : `Refreshed ${refreshed}; ${failed} failed`);
        });
    };

    const checkNetworkEndpoints = () => {
        props.close();
        void (async () => {
            try {
                const networksResponse = await api("/net", "get", {});

                if (networksResponse.status !== 200) throw new Error("Failed to load networks");

                const endpointResponses = await Promise.all(networksResponse.data.networks.map(network => api(
                    "/net/{network_identity}/endpoints",
                    "get",
                    { path: { network_identity: network.network_identity } },
                )));
                const endpoints = endpointResponses.flatMap(response => (response.status === 200 ? response.data : []));
                const results = await Promise.allSettled(endpoints.map(endpoint => api(
                    "/net/{network_identity}/endpoints/{endpoint_identity}/status",
                    "get",
                    {
                        path: {
                            network_identity: endpoint.network_identity,
                            endpoint_identity: endpoint.endpoint_identity,
                        },
                    },
                )));
                const statuses = results.flatMap(result => (
                    result.status === "fulfilled" && result.value.status === 200
                        ? [result.value.data.status]
                        : []
                ));
                const alive = statuses.filter(status => status === "Alive").length;
                const dead = statuses.filter(status => status === "Dead").length;
                const disabled = statuses.filter(status => status === "Disabled").length;
                const failed = results.length - statuses.length;
                const failures = failed > 0 ? `, ${failed} failed` : "";

                showToast(`${alive} alive, ${dead} dead, ${disabled} disabled${failures}`);
            }
            catch {
                showToast("Failed to check network endpoints");
            }
        })();
    };

    return (
        <Command.Group heading="Diagnostics">
            <Show when={(layoutQuery.data?.accounts.length ?? 0) > 0}>
                <CommandMenuItem
                  value="refresh all account balances"
                  keywords={["reload", "sync", "update", "wallets"]}
                  icon={FaSolidRefresh}
                  title="Refresh all balances"
                  description="Fetch fresh balances for every account"
                  onSelect={refreshAllBalances}
                />
            </Show>
            <CommandMenuItem
              value="check network endpoints"
              keywords={["rpc", "status", "health", "diagnostics"]}
              icon={FiActivity}
              title="Check network endpoints"
              description="Probe every configured RPC endpoint"
              onSelect={checkNetworkEndpoints}
            />
        </Command.Group>
    );
};
