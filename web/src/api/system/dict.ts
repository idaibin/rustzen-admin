import { apiRequest, proTableRequest } from "../request";
import type { OptionItem } from "Api";
import type { Dict } from "System";

/**
 * 字典管理API服务
 */
export const dictAPI = {
    getTableData: (params?: Dict.QueryParams) =>
        proTableRequest<Dict.Item, Dict.QueryParams>({
            url: "/api/system/dicts",
            params,
        }),

    create: (data: Dict.CreateRequest) =>
        apiRequest<Dict.Item, Dict.CreateRequest>({
            url: "/api/system/dicts",
            method: "POST",
            params: data,
        }),

    update: (id: number, data: Dict.UpdateRequest) =>
        apiRequest<Dict.Item, Dict.UpdateRequest>({
            url: `/api/system/dicts/${id}`,
            method: "PUT",
            params: data,
        }),

    delete: (id: number) =>
        apiRequest<void, Dict.QueryParams>({
            url: `/api/system/dicts/${id}`,
            method: "DELETE",
        }),

    getOptions: () =>
        apiRequest<OptionItem[]>({ url: "/api/system/dicts/options" }),

    getOptionsByType: (type: string) =>
        apiRequest<Dict.Item[]>({ url: `/api/system/dicts/type/${type}` }),
};
