import { apiRequest } from "@/api/request";

/**
 * Dictionary management API service.
 */
export const dictAPI = {
    list: async (params: Dict.QueryParams) => {
        return apiRequest({
            url: "/api/system/dicts",
            params,
            raw: true,
        });
    },
    create: (data: Dict.CreateRequest) => {
        return apiRequest<Dict.Item, Dict.CreateRequest>({
            url: "/api/system/dicts",
            method: "POST",
            params: data,
        });
    },
    update: (id: number, data: Dict.UpdateRequest) => {
        return apiRequest<Dict.Item, Dict.UpdateRequest>({
            url: `/api/system/dicts/${id}`,
            method: "PUT",
            params: data,
        });
    },
    delete: (id: number) => {
        return apiRequest<void>({
            url: `/api/system/dicts/${id}`,
            method: "DELETE",
        });
    },
    options: () => {
        return apiRequest<Api.OptionItem<string>[]>({
            url: "/api/system/dicts/options",
        });
    },
    byType: (type: string) => {
        return apiRequest<Api.OptionItem<string>[]>({
            url: `/api/system/dicts/type/${type}`,
        });
    },
};
