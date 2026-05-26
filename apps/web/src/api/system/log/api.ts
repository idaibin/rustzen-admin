import { apiDownload, apiRequest } from "@/api/request";

/**
 * Log management API service.
 */
export const logAPI = {
    list: async (params: Log.QueryParams) => {
        const res = await apiRequest<Log.Item[], Log.QueryParams>({
            url: "/api/system/logs",
            params,
            raw: true,
        });
        return {
            data: res.data,
            total: res.total ?? 0,
            success: true,
        };
    },
    export: () => {
        return apiDownload({ url: "/api/system/logs/export" });
    },
};
