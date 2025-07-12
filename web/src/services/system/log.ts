import { proTableRequest, request } from "../api";
import type { Log } from "System";

/**
 * 日志管理API服务
 */
export const logAPI = {
    // 完整请求方法
    getLogList: (params?: Log.QueryParams) =>
        proTableRequest<Log.Item>("/api/system/logs", params),

    getLogById: (id: number) => request.get<Log.Item>(`/api/system/logs/${id}`),

    deleteLog: (id: number) => request.del<void>(`/api/system/logs/${id}`),

    clearLogs: () => request.del<void>("/api/system/logs/clear"),

    exportLogs: (params?: Log.QueryParams) =>
        request.get<Blob>("/api/system/logs/export", params),

    // URL生成器（SWR使用）
    urls: {
        getLogById: (id: number) => `/api/system/logs/${id}`,
        getLogList: () => "/api/system/logs",
    },
};
