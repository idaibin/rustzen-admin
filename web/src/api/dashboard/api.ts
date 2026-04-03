import { apiRequest } from "@/api/request";

export const dashboardAPI = {
    stats: () => {
        return apiRequest<Dashboard.Stats>({ url: "/api/dashboard/stats" });
    },
    health: () => {
        return apiRequest<Dashboard.SystemHealth>({
            url: "/api/dashboard/health",
        });
    },
    metrics: () => {
        return apiRequest<Dashboard.SystemMetricsData>({
            url: "/api/dashboard/metrics",
        });
    },
    trends: () => {
        return apiRequest<Dashboard.UserActivityChart>({
            url: "/api/dashboard/trends",
        });
    },
};
