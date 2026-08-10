import { queryOptions } from "@tanstack/react-query";

import { insightsAPI } from "./api";

export const insightsQueryKeys = {
    collectionPolicy: () => ["insights", "collection-policy"] as const,
};

export const insightsQueryOptions = {
    collectionPolicy: () =>
        queryOptions({
            queryKey: insightsQueryKeys.collectionPolicy(),
            queryFn: insightsAPI.collectionPolicy,
            staleTime: 30_000,
        }),
};
