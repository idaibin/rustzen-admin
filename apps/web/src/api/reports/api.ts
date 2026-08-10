import { routePath } from "@/api/module-contract";
import { reportsAPIContract as contract } from "@/api/reports/contract";
import { apiBlob, apiDownload, apiRequest } from "@/api/request";

export const reportsAPI = {
    settings: () =>
        apiRequest<Reports.InstallationSettings>({
            url: contract.settings.path,
            method: contract.settings.method,
        }),
    systems: () => apiRequest<Reports.System[]>({ url: contract.systems.path }),
    createSystem: (params: Reports.SaveSystem) =>
        apiRequest<Reports.System, Reports.SaveSystem>({
            url: contract.createSystem.path,
            method: contract.createSystem.method,
            params,
        }),
    flows: (systemId?: string) =>
        apiRequest<Reports.Flow[]>({ url: contract.flows.path, params: { systemId } }),
    createFlow: (params: Reports.SaveFlow) =>
        apiRequest<Reports.Flow, Reports.SaveFlow>({
            url: contract.createFlow.path,
            method: contract.createFlow.method,
            params,
        }),
    updateFlow: (id: string, params: Reports.SaveFlow) =>
        apiRequest<Reports.Flow, Reports.SaveFlow>({
            url: routePath(contract.updateFlow, { id }),
            method: contract.updateFlow.method,
            params,
        }),
    deleteFlow: (id: string) =>
        apiRequest<void>({
            url: routePath(contract.deleteFlow, { id }),
            method: contract.deleteFlow.method,
        }),
    flowOptions: () =>
        apiRequest<Reports.FlowOption[]>({
            url: contract.flowOptions.path,
            method: contract.flowOptions.method,
        }),
    schedules: () => apiRequest<Reports.Schedule[]>({ url: contract.schedules.path }),
    createSchedule: (params: Reports.SaveSchedule) =>
        apiRequest<Reports.Schedule, Reports.SaveSchedule>({
            url: contract.createSchedule.path,
            method: contract.createSchedule.method,
            params,
        }),
    schedule: (id: string) =>
        apiRequest<Reports.Schedule>({ url: routePath(contract.schedule, { id }) }),
    updateSchedule: (id: string, params: Reports.SaveSchedule) =>
        apiRequest<Reports.Schedule, Reports.SaveSchedule>({
            url: routePath(contract.updateSchedule, { id }),
            method: contract.updateSchedule.method,
            params,
        }),
    deleteSchedule: (id: string) =>
        apiRequest<void>({
            url: routePath(contract.deleteSchedule, { id }),
            method: contract.deleteSchedule.method,
        }),
    runs: (params: { current?: number; pageSize?: number; status?: string }) =>
        apiRequest<Reports.Page<Reports.Run>, typeof params>({
            url: contract.runs.path,
            params,
        }),
    run: (id: string) => apiRequest<Reports.Run>({ url: routePath(contract.run, { id }) }),
    createRun: (params: Reports.CreateRun) =>
        apiRequest<Reports.Run, Reports.CreateRun>({
            url: contract.createRun.path,
            method: contract.createRun.method,
            params,
        }),
    cancelRun: (id: string) =>
        apiRequest<Reports.Run>({
            url: routePath(contract.cancelRun, { id }),
            method: contract.cancelRun.method,
        }),
    runSteps: (id: string) =>
        apiRequest<Reports.RunStep[]>({ url: routePath(contract.runSteps, { id }) }),
    runArtifacts: (id: string) =>
        apiRequest<Reports.Artifact[]>({ url: routePath(contract.runArtifacts, { id }) }),
    downloadArtifact: (runId: string, artifactId: string, filename: string) =>
        apiDownload({
            url: routePath(contract.artifact, { run_id: runId, artifact_id: artifactId }),
            filename,
        }),
    liveFrame: async (id: string, signal?: AbortSignal) => {
        const blob = await apiBlob({ url: routePath(contract.liveFrame, { id }), signal });
        if (blob && (blob.type !== "image/png" || blob.size === 0)) {
            throw new Error("Invalid live frame response");
        }
        return blob;
    },
};
