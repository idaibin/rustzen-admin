import { proTableRequest } from "@/api";

/**
 * 日志管理API服务
 */
export const logAPI = {
    getTableData: (params?: Log.QueryParams) =>
        proTableRequest<Log.Item, Log.QueryParams>({
            url: "/api/system/logs",
            params,
        }),
};
