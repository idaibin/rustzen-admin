import { apiRequest, proTableRequest } from "@/api";

/**
 * 角色管理API服务
 */
export const roleAPI = {
    listRoles: (params?: Role.QueryParams) =>
        proTableRequest<Role.Item, Role.QueryParams>({
            url: "/api/system/roles",
            params,
        }),

    createRole: (data: Role.CreateRequest) =>
        apiRequest<Role.Item, Role.CreateRequest>({
            url: "/api/system/roles",
            method: "POST",
            params: data,
        }),

    updateRole: (id: number, data: Role.UpdateRequest) =>
        apiRequest<Role.Item, Role.UpdateRequest>({
            url: `/api/system/roles/${id}`,
            method: "PUT",
            params: data,
        }),

    deleteRole: (id: number) => apiRequest<void>({ url: `/api/system/roles/${id}`, method: "DELETE" }),

    listRoleOptions: () => apiRequest<Api.OptionItem<number>[]>({ url: "/api/system/roles/options" }),
};
