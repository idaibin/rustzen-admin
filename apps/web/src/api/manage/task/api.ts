import { apiRequest } from "@/api/request";

export const taskAPI = {
    list: async () => {
        const res = await apiRequest<Task.Item[]>({
            url: "/api/manage/tasks",
            raw: true,
        });
        return {
            data: res.data,
            total: res.total ?? res.data.length,
            success: true,
        };
    },
    runs: async (taskKey: string, params?: Task.RunQuery) => {
        const res = await apiRequest<Task.RunItem[], Task.RunQuery>({
            url: `/api/manage/tasks/${taskKey}/runs`,
            params,
            raw: true,
        });
        return {
            data: res.data,
            total: res.total ?? 0,
            success: true,
        };
    },
    run: (taskKey: string) => {
        return apiRequest<Task.RunItem>({
            url: `/api/manage/tasks/${taskKey}/run`,
            method: "POST",
        });
    },
};
