import { apiRequest } from "@/api/request";

/**
 * Dictionary management API service.
 */
export const dictAPI = {
    list: async (params: Dict.QueryParams) => {
        const res = await apiRequest<Dict.Item[], Dict.QueryParams>({
            url: "/api/manage/dicts",
            params,
            raw: true,
        });
        return {
            data: res.data,
            total: res.total ?? 0,
            success: true,
        };
    },
    create: (data: Dict.CreateRequest) => {
        return apiRequest<number, Dict.CreateRequest>({
            url: "/api/manage/dicts",
            method: "POST",
            params: data,
        });
    },
    update: (id: number, data: Dict.UpdateRequest) => {
        return apiRequest<number, Dict.UpdateRequest>({
            url: `/api/manage/dicts/${id}`,
            method: "PUT",
            params: data,
        });
    },
    delete: (id: number) => {
        return apiRequest<void>({
            url: `/api/manage/dicts/${id}`,
            method: "DELETE",
        });
    },
    options: () => {
        return apiRequest<Api.OptionItem<string>[]>({
            url: "/api/manage/dicts/options",
        });
    },
    status: (id: number, status: number) => {
        return apiRequest<void>({
            url: `/api/manage/dicts/${id}/status`,
            method: "PATCH",
            params: { status },
        });
    },
    byType: (type: string) => {
        return apiRequest<Api.OptionItem<string>[]>({
            url: `/api/manage/dicts/type/${type}`,
        });
    },
};
