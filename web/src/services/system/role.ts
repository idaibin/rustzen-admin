import { proTableRequest, request } from "../api";
import type { OptionItem } from "Api";
import type { Role } from "System";

/**
 * 角色管理API服务
 */
export const roleAPI = {
    // 完整请求方法
    getRoleList: (params?: Role.QueryParams) =>
        proTableRequest<Role.Item>("/api/system/roles", params),

    getRoleById: (id: number) =>
        request.get<Role.Item>(`/api/system/roles/${id}`),

    createRole: (data: Role.CreateRequest) =>
        request.post<Role.Item, Role.CreateRequest>("/api/system/roles", data),

    updateRole: (id: number, data: Role.UpdateRequest) =>
        request.put<Role.Item, Role.UpdateRequest>(
            `/api/system/roles/${id}`,
            data
        ),

    deleteRole: (id: number) => request.del<void>(`/api/system/roles/${id}`),

    getRoleMenus: (id: number) =>
        request.get<number[]>(`/api/system/roles/${id}/menus`),

    setRoleMenus: (id: number, menuIds: number[]) =>
        request.put<void, number[]>(`/api/system/roles/${id}/menus`, menuIds),

    getRoleOptions: () =>
        request.get<OptionItem[]>("/api/system/roles/options"),

    // URL生成器（SWR使用）
    urls: {
        getRoleById: (id: number) => `/api/system/roles/${id}`,
        getRoleList: () => "/api/system/roles",
        getRoleMenus: (id: number) => `/api/system/roles/${id}/menus`,
        getRoleOptions: () => "/api/system/roles/options",
    },
};
