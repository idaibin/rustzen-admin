import { routePath } from "@/api/module-contract";
import { monitorAPIContract as contract } from "@/api/monitor/contract";
import { apiRequest } from "@/api/request";

export const monitorAPI = {
    overview: () => apiRequest<Monitor.Overview>({ url: contract.overview.path }),
    nodes: () => apiRequest<Monitor.Node[]>({ url: contract.nodes.path }),
    node: (nodeId: string) =>
        apiRequest<Monitor.Node>({ url: routePath(contract.node, { node_id: nodeId }) }),
    metrics: (nodeId: string, params: Monitor.MetricsQuery = {}) =>
        apiRequest<Monitor.Metrics, Monitor.MetricsQuery>({
            url: routePath(contract.metrics, { node_id: nodeId }),
            params,
        }),
    nodeAlertSettings: (nodeId: string) =>
        apiRequest<Monitor.AlertSettings>({
            url: routePath(contract.nodeAlertSettings, { node_id: nodeId }),
        }),
    updateNodeAlertSettings: (nodeId: string, params: Monitor.UpdateAlertSettings) =>
        apiRequest<Monitor.AlertSettings, Monitor.UpdateAlertSettings>({
            url: routePath(contract.updateNodeAlertSettings, { node_id: nodeId }),
            method: contract.updateNodeAlertSettings.method,
            params,
        }),
    resetNodeAlertSettings: (nodeId: string) =>
        apiRequest<Monitor.AlertSettings>({
            url: routePath(contract.resetNodeAlertSettings, { node_id: nodeId }),
            method: contract.resetNodeAlertSettings.method,
        }),
    incidents: (params: Monitor.IncidentQuery = {}) =>
        apiRequest<Monitor.Page<Monitor.IncidentSummary>, Monitor.IncidentQuery>({
            url: contract.incidents.path,
            params,
        }),
    incident: (id: string) =>
        apiRequest<Monitor.IncidentDetail>({ url: routePath(contract.incident, { id }) }),
    alertSettings: () => apiRequest<Monitor.AlertSettings>({ url: contract.alertSettings.path }),
    updateAlertSettings: (params: Monitor.UpdateAlertSettings) =>
        apiRequest<Monitor.AlertSettings, Monitor.UpdateAlertSettings>({
            url: contract.updateAlertSettings.path,
            method: contract.updateAlertSettings.method,
            params,
        }),
    dailySummaries: (params: Monitor.DailySummaryQuery = {}) =>
        apiRequest<Monitor.Page<Monitor.DailySummary>, Monitor.DailySummaryQuery>({
            url: contract.dailySummaries.path,
            params,
        }),
};
