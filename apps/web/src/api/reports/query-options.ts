import { queryOptions } from "@tanstack/react-query";

import { reportsCoreAPI as reportsAPI } from "./core-api";

export const reportsQueryKeys = {
    settings: () => ["reports", "settings"] as const,
    systems: () => ["reports", "systems"] as const,
    flows: () => ["reports", "flows"] as const,
    flowOptions: () => ["reports", "flow-options"] as const,
    schedules: () => ["reports", "schedules"] as const,
};

export const reportsQueryOptions = {
    settings: () =>
        queryOptions({
            queryKey: reportsQueryKeys.settings(),
            queryFn: reportsAPI.settings,
        }),
    systems: () =>
        queryOptions({
            queryKey: reportsQueryKeys.systems(),
            queryFn: reportsAPI.systems,
        }),
    flows: () =>
        queryOptions({
            queryKey: reportsQueryKeys.flows(),
            queryFn: () => reportsAPI.flows(),
        }),
    flowOptions: () =>
        queryOptions({
            queryKey: reportsQueryKeys.flowOptions(),
            queryFn: reportsAPI.flowOptions,
        }),
    schedules: () =>
        queryOptions({
            queryKey: reportsQueryKeys.schedules(),
            queryFn: reportsAPI.schedules,
        }),
};
