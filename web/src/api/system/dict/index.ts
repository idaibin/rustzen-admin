import { apiRequest, proTableRequest } from "@/api";

/**
 * 字典管理API服务
 */
export const dictAPI = {
    listDicts: (params?: Dict.QueryParams) =>
        proTableRequest<Dict.Item, Dict.QueryParams>({
            url: "/api/system/dicts",
            params,
        }),

    createDict: (data: Dict.CreateRequest) =>
        apiRequest<Dict.Item, Dict.CreateRequest>({
            url: "/api/system/dicts",
            method: "POST",
            params: data,
        }),

    updateDict: (id: number, data: Dict.UpdateRequest) =>
        apiRequest<Dict.Item, Dict.UpdateRequest>({
            url: `/api/system/dicts/${id}`,
            method: "PUT",
            params: data,
        }),

    deleteDict: (id: number) =>
        apiRequest<void>({
            url: `/api/system/dicts/${id}`,
            method: "DELETE",
        }),

    listDictOptions: () => apiRequest<Api.OptionItem<string>[]>({ url: "/api/system/dicts/options" }),

    listDictsByType: (type: string) =>
        apiRequest<Api.OptionItem<string>[]>({ url: `/api/system/dicts/type/${type}` }),
};
