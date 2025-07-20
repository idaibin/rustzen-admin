import { proTableRequest, request } from "../api";
import type { OptionItem } from "Api";
import type { Menu } from "System";

/**
 * 菜单管理API服务
 */
export const menuAPI = {
    // 完整请求方法
    getMenuList: (params?: Menu.QueryParams) =>
        proTableRequest<Menu.Item>("/api/system/menus", params),

    getMenuById: (id: number) =>
        request.get<Menu.Item>(`/api/system/menus/${id}`),

    createMenu: (data: Menu.CreateAndUpdateRequest) =>
        request.post<Menu.Item, Menu.CreateAndUpdateRequest>(
            "/api/system/menus",
            data
        ),

    updateMenu: (id: number, data: Menu.CreateAndUpdateRequest) =>
        request.put<Menu.Item, Menu.CreateAndUpdateRequest>(
            `/api/system/menus/${id}`,
            data
        ),

    deleteMenu: (id: number) => request.del<void>(`/api/system/menus/${id}`),

    getMenuTree: () => request.get<Menu.Item[]>("/api/system/menus/tree"),

    getMenuOptions: () =>
        request.get<OptionItem[]>("/api/system/menus/options"),

    // URL生成器（SWR使用）
    urls: {
        getMenuById: (id: number) => `/system/menus/${id}`,
        getMenuList: () => "/api/system/menus",
        getMenuTree: () => "/api/system/menus/tree",
        getMenuOptions: () => "/api/system/menus/options",
    },
};
