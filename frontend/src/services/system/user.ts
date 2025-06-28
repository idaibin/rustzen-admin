import { proTableRequest, request } from "../api";
import type { OptionItem } from "Api";
import type { User } from "System";

/**
 * 用户管理API服务
 */
export const userAPI = {
  // 完整请求方法
  getUserList: (params?: User.QueryParams) =>
    proTableRequest<User.Item>("/system/users", params),

  getUserById: (id: number) => request.get<User.Item>(`/system/users/${id}`),

  createUser: (data: User.CreateRequest) =>
    request.post<User.Item, User.CreateRequest>("/system/users", data),

  updateUser: (id: number, data: User.UpdateRequest) =>
    request.put<User.Item, User.UpdateRequest>(`/system/users/${id}`, data),

  deleteUser: (id: number) => request.del<void>(`/system/users/${id}`),

  getUserOptions: (params?: { status?: string; q?: string; limit?: number }) =>
    request.get<OptionItem[], { status?: string; q?: string; limit?: number }>(
      "/system/users/options",
      params
    ),

  getUserStatusOptions: () =>
    request.get<OptionItem[]>("/system/users/status-options"),

  // URL生成器（SWR使用）
  urls: {
    getUserById: (id: number) => `/system/users/${id}`,
    getUserList: () => "/system/users",
    getUserOptions: () => "/system/users/options",
    getUserStatusOptions: () => "/system/users/status-options",
  },
};
