import { apiDownload, proTableRequest } from "@/api";

/**
 * 日志管理API服务
 */
export const logAPI = {
    listLogs: (params?: Log.QueryParams) =>
        proTableRequest<Log.Item, Log.QueryParams>({
            url: "/api/system/logs",
            params,
        }),

    exportLogs: () => apiDownload({ url: "/api/system/logs/export" }),
};
