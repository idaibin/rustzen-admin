import type { ModuleApiRoute } from "@/api/module-contract";

export const monitorAPIContract = {
    overview: { method: "GET", path: "/api/monitor/overview" },
    nodes: { method: "GET", path: "/api/monitor/nodes" },
    node: { method: "GET", path: "/api/monitor/nodes/{node_id}" },
    metrics: { method: "GET", path: "/api/monitor/nodes/{node_id}/metrics" },
    checks: { method: "GET", path: "/api/monitor/checks" },
    check: { method: "GET", path: "/api/monitor/checks/{id}" },
    checkResults: { method: "GET", path: "/api/monitor/checks/{id}/results" },
    incidents: { method: "GET", path: "/api/monitor/incidents" },
    incident: { method: "GET", path: "/api/monitor/incidents/{id}" },
    createCheck: { method: "POST", path: "/api/monitor/checks" },
    updateCheck: { method: "PUT", path: "/api/monitor/checks/{id}" },
    deleteCheck: { method: "DELETE", path: "/api/monitor/checks/{id}" },
    setCheckEnabled: { method: "PUT", path: "/api/monitor/checks/{id}/enabled" },
    testCheck: { method: "POST", path: "/api/monitor/checks/test" },
} as const satisfies Record<string, ModuleApiRoute>;
