import { apiRequest } from "@/api/request";

/**
 * Menu management API service.
 */
export const menuAPI = {
    list: async (params: Menu.QueryParams) => {
        const res = await apiRequest<Menu.Item[], Menu.QueryParams>({
            url: "/api/system/menus",
            params,
            raw: true,
        });
        const tree = buildMenuTree(res.data);
        return {
            data: tree,
            total: res.total ?? tree.length,
            success: true,
        };
    },
    create: (data: Menu.CreateRequest) => {
        return apiRequest<Menu.Item, Menu.CreateRequest>({
            url: "/api/system/menus",
            method: "POST",
            params: data,
        });
    },
    update: (id: number, data: Menu.UpdateRequest) => {
        return apiRequest<Menu.Item, Menu.UpdateRequest>({
            url: `/api/system/menus/${id}`,
            method: "PUT",
            params: data,
        });
    },
    delete: (id: number) => {
        return apiRequest<void>({
            url: `/api/system/menus/${id}`,
            method: "DELETE",
        });
    },
    options: async () => {
        const res = await apiRequest<Api.OptionItem<number>[]>({
            url: "/api/system/menus/options",
        });
        return [{ label: "Root", value: 0 }, ...res];
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
