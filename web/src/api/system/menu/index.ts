import { apiRequest, proTableRequest } from "@/api";

/**
 * 菜单管理API服务
 */
export const menuAPI = {
    listMenus: (params?: Menu.QueryParams) => {
        return proTableRequest<Menu.Item, Menu.QueryParams>({
            url: "/api/system/menus",
            params,
        }).then((res) => {
            return {
                ...res,
                data: buildMenuTree(res.data),
            };
        });
    },

    createMenu: (data: Menu.CreateRequest) =>
        apiRequest<Menu.Item, Menu.CreateRequest>({
            url: "/api/system/menus",
            method: "POST",
            params: data,
        }),

    updateMenu: (id: number, data: Menu.UpdateRequest) =>
        apiRequest<Menu.Item, Menu.UpdateRequest>({
            url: `/api/system/menus/${id}`,
            method: "PUT",
            params: data,
        }),

    deleteMenu: (id: number) => apiRequest<void>({ url: `/api/system/menus/${id}`, method: "DELETE" }),

    listMenuOptions: () =>
        apiRequest<Api.OptionItem<number>[]>({ url: "/api/system/menus/options" }).then((res) => [
            { label: "Root", value: 0 },
            ...res,
        ]),
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
