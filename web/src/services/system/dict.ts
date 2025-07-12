import { proTableRequest, request } from "../api";
import type { OptionItem } from "Api";
import type { Dict } from "System";

/**
 * 字典管理API服务
 */
export const dictAPI = {
    // 完整请求方法
    getDictList: (params?: Dict.QueryParams) =>
        proTableRequest<Dict.Item>("/api/system/dicts", params),

    getDictById: (id: number) =>
        request.get<Dict.Item>(`/api/system/dicts/${id}`),

    createDict: (data: Dict.CreateRequest) =>
        request.post<Dict.Item, Dict.CreateRequest>("/api/system/dicts", data),

    updateDict: (id: number, data: Dict.UpdateRequest) =>
        request.put<Dict.Item, Dict.UpdateRequest>(
            `/api/system/dicts/${id}`,
            data
        ),

    deleteDict: (id: number) => request.del<void>(`/api/system/dicts/${id}`),

    getDictByCode: (code: string) =>
        request.get<Dict.Item[]>(`/api/system/dicts/code/${code}`),

    getDictOptions: () =>
        request.get<OptionItem[]>("/api/system/dicts/options"),

    // URL生成器（SWR使用）
    urls: {
        getDictById: (id: number) => `/api/system/dicts/${id}`,
        getDictList: () => "/api/system/dicts",
        getDictByCode: (code: string) => `/api/system/dicts/code/${code}`,
        getDictOptions: () => "/api/system/dicts/options",
    },
};
