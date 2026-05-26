import { apiRequest } from "@/api/request";

/**
 * Role management API service.
 */
export const roleAPI = {
    list: async (params: Role.QueryParams) => {
        const res = await apiRequest<Role.Item[], Role.QueryParams>({
            url: "/api/system/roles",
            params,
            raw: true,
        });
        return {
            data: res.data,
            total: res.total ?? 0,
            success: true,
        };
    },
    create: (data: Role.CreateRequest) => {
        return apiRequest<Role.Item, Role.CreateRequest>({
            url: "/api/system/roles",
            method: "POST",
            params: data,
        });
    },
    update: (id: number, data: Role.UpdateRequest) => {
        return apiRequest<Role.Item, Role.UpdateRequest>({
            url: `/api/system/roles/${id}`,
            method: "PUT",
            params: data,
        });
    },
    delete: (id: number) => {
        return apiRequest<void>({
            url: `/api/system/roles/${id}`,
            method: "DELETE",
        });
    },
    options: () => {
        return apiRequest<Api.OptionItem<number>[]>({
            url: "/api/system/roles/options",
        });
    },
};
