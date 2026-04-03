import { apiDownload, apiRequest } from "@/api/request";

/**
 * Log management API service.
 */
export const logAPI = {
    list: async (params: Log.QueryParams) => {
        return apiRequest({
            url: "/api/system/logs",
            params,
            raw: true,
        });
    },
    export: () => {
        return apiDownload({ url: "/api/system/logs/export" });
    },
};
