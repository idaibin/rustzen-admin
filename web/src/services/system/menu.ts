import { proTableRequest, request } from "../api";
import type { OptionItem } from "Api";
import type { Menu } from "System";

/**
 * 菜单管理API服务
 */
export const menuAPI = {
    getMenuList: (params?: Menu.QueryParams) => {
        return proTableRequest<Menu.Item>("/api/system/menus", params).then(
            (res) => {
                res.data = buildMenuTree(res.data);
                console.log("res.data", res.data);
                return res;
            }
        );
    },

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

function buildMenuTree(list: Menu.Item[], parentId = 0): Menu.Item[] {
    return list
        .filter((item) => item.parentId === parentId)
        .map((item) => {
            const child = buildMenuTree(list, item.id);
            return {
                ...item,
                children: child.length > 0 ? child : null,
            };
        });
}
