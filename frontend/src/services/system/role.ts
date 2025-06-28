import { proTableRequest, request } from "../api";
import type { OptionItem } from "Api";
import type { Role } from "System";

/**
 * 角色管理API服务
 */
export const roleAPI = {
  // 完整请求方法
  getRoleList: (params?: Role.QueryParams) =>
    proTableRequest<Role.Item>("/system/roles", params),

  getRoleById: (id: number) => request.get<Role.Item>(`/system/roles/${id}`),

  createRole: (data: Role.CreateRequest) =>
    request.post<Role.Item, Role.CreateRequest>("/system/roles", data),

  updateRole: (id: number, data: Role.UpdateRequest) =>
    request.put<Role.Item, Role.UpdateRequest>(`/system/roles/${id}`, data),

  deleteRole: (id: number) => request.del<void>(`/system/roles/${id}`),

  getRoleMenus: (id: number) =>
    request.get<number[]>(`/system/roles/${id}/menus`),

  setRoleMenus: (id: number, menuIds: number[]) =>
    request.put<void, number[]>(`/system/roles/${id}/menus`, menuIds),

  getRoleOptions: () => request.get<OptionItem[]>("/system/roles/options"),

  // URL生成器（SWR使用）
  urls: {
    getRoleById: (id: number) => `/system/roles/${id}`,
    getRoleList: () => "/system/roles",
    getRoleMenus: (id: number) => `/system/roles/${id}/menus`,
    getRoleOptions: () => "/system/roles/options",
  },
};
