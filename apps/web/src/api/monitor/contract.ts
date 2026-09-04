import type { ModuleApiRoute } from "@/api/module-contract";

export const monitorAPIContract = {
    overview: { method: "GET", path: "/api/monitor/overview" },
    nodes: { method: "GET", path: "/api/monitor/nodes" },
    node: { method: "GET", path: "/api/monitor/nodes/{node_id}" },
    metrics: { method: "GET", path: "/api/monitor/nodes/{node_id}/metrics" },
    nodeAlertSettings: { method: "GET", path: "/api/monitor/nodes/{node_id}/alert-settings" },
    updateNodeAlertSettings: {
        method: "PUT",
        path: "/api/monitor/nodes/{node_id}/alert-settings",
    },
    resetNodeAlertSettings: {
        method: "DELETE",
        path: "/api/monitor/nodes/{node_id}/alert-settings",
    },
    incidents: { method: "GET", path: "/api/monitor/incidents" },
    incident: { method: "GET", path: "/api/monitor/incidents/{id}" },
    alertSettings: { method: "GET", path: "/api/monitor/alert-settings" },
    updateAlertSettings: { method: "PUT", path: "/api/monitor/alert-settings" },
    dailySummaries: { method: "GET", path: "/api/monitor/daily-summaries" },
} as const satisfies Record<string, ModuleApiRoute>;
