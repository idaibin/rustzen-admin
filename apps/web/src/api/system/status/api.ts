import { apiRequest } from "@/api/request";

import { moduleLogAPI } from "./module-logs";

export const statusAPI = {
    overview: () => {
        return apiRequest<SystemStatus.Overview>({
            url: "/api/system/status",
        });
    },
    moduleLogs: moduleLogAPI,
};
