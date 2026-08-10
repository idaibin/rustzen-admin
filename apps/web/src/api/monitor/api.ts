import { routePath } from "@/api/module-contract";
import { monitorAPIContract as contract } from "@/api/monitor/contract";
import { apiRequest } from "@/api/request";

export const monitorAPI = {
    overview: () => apiRequest<Monitor.Overview>({ url: contract.overview.path }),
    nodes: () => apiRequest<Monitor.Node[]>({ url: contract.nodes.path }),
    metrics: (nodeId: string, params: Monitor.MetricsQuery = {}) =>
        apiRequest<Monitor.MetricPoint[], Monitor.MetricsQuery>({
            url: routePath(contract.metrics, { node_id: nodeId }),
            params,
        }),
    checks: (params: Monitor.CheckQuery = {}) =>
        apiRequest<Monitor.Page<Monitor.Check>, Monitor.CheckQuery>({
            url: contract.checks.path,
            params,
        }),
    incidents: (params: Monitor.IncidentQuery = {}) =>
        apiRequest<Monitor.Page<Monitor.IncidentSummary>, Monitor.IncidentQuery>({
            url: contract.incidents.path,
            params,
        }),
    incident: (id: string) =>
        apiRequest<Monitor.IncidentDetail>({
            url: routePath(contract.incident, { id }),
        }),
    createCheck: (params: Monitor.SaveCheck) =>
        apiRequest<Monitor.Check, Monitor.SaveCheck>({
            url: contract.createCheck.path,
            method: contract.createCheck.method,
            params,
        }),
    updateCheck: (id: string, params: Monitor.SaveCheck) =>
        apiRequest<Monitor.Check, Monitor.SaveCheck>({
            url: routePath(contract.updateCheck, { id }),
            method: contract.updateCheck.method,
            params,
        }),
    deleteCheck: (id: string) =>
        apiRequest<void>({
            url: routePath(contract.deleteCheck, { id }),
            method: contract.deleteCheck.method,
        }),
    setCheckEnabled: (id: string, enabled: boolean) =>
        apiRequest<Monitor.Check, { enabled: boolean }>({
            url: routePath(contract.setCheckEnabled, { id }),
            method: contract.setCheckEnabled.method,
            params: { enabled },
        }),
    testCheck: (params: Pick<Monitor.SaveCheck, "host" | "port" | "timeoutMs">) =>
        apiRequest<Monitor.ProbeResult, Pick<Monitor.SaveCheck, "host" | "port" | "timeoutMs">>({
            url: contract.testCheck.path,
            method: contract.testCheck.method,
            params,
        }),
};
