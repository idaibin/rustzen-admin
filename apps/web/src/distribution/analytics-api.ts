import { apiRequest } from "@/api/request";
import {
    accountAPI,
    appMessage,
    authAPI,
    installationAPI,
    MessageContent,
    systemAPI,
} from "@/distribution/portal-api";

const insightsAPI = {
    overview: (params: Insights.OverviewQuery) =>
        apiRequest<Insights.Overview, Insights.OverviewQuery>({
            url: "/api/insights/overview",
            method: "GET",
            params,
            silent: true,
        }),
    events: (params: Insights.EventQuery) =>
        apiRequest<Insights.Page<Insights.Event>, Insights.EventQuery>({
            url: "/api/insights/events",
            method: "GET",
            params,
            silent: true,
        }),
};

export { accountAPI, appMessage, authAPI, installationAPI, MessageContent, insightsAPI, systemAPI };
