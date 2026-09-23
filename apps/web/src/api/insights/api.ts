import { insightsAPIContract as contract } from "@/api/insights/contract";
import { apiRequest } from "@/api/request";

export const insightsAPI = {
    overview: (params: Insights.OverviewQuery) =>
        apiRequest<Insights.Overview, Insights.OverviewQuery>({
            url: contract.overview.path,
            method: contract.overview.method,
            params,
            // Route-local DataState needs the typed HTTP status and owns the
            // visible failure instead of also emitting a global toast.
            silent: true,
        }),
    events: (params: Insights.EventQuery) =>
        apiRequest<Insights.Page<Insights.Event>, Insights.EventQuery>({
            url: contract.events.path,
            method: contract.events.method,
            params,
            // Keep the same typed, route-local error contract as overview.
            silent: true,
        }),
};
