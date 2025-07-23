import { proTableRequest } from "../request";
import type { Log } from "System";

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
