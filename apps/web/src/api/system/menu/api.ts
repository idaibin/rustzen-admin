import { apiRequest } from "@/api/request";
import { t } from "@/lib/i18n";

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
    inventory: () => {
        return apiRequest<Menu.Item[]>({
            url: "/api/system/menus/inventory",
        });
    },
    update: (id: number, data: Menu.UpdateRequest) => {
        return apiRequest<number, Menu.UpdateRequest>({
            url: `/api/system/menus/inventory/${id}`,
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
        const res = await apiRequest<Menu.OptionItem[]>({
            url: "/api/system/menus/options",
        });
        return [
            {
                label: t("根目录", "Root"),
                value: 0,
                code: "",
                isSystem: true,
                moduleId: null,
                moduleMenuCode: null,
            },
            ...res,
        ];
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
