import { apiRequest, proTableRequest } from "../request";
import type { OptionItem } from "Api";
import type { User } from "System";

/**
 * 用户管理API服务
 */
export const userAPI = {
    getTableData: (params?: User.QueryParams) =>
        proTableRequest<User.Item, User.QueryParams>({
            url: "/api/system/users",
            params,
        }),

    create: (data: User.CreateRequest) =>
        apiRequest<User.Item, User.CreateRequest>({
            url: "/api/system/users",
            method: "POST",
            params: data,
        }),

    update: (id: number, data: User.UpdateRequest) =>
        apiRequest<User.Item, User.UpdateRequest>({
            url: `/api/system/users/${id}`,
            method: "PUT",
            params: data,
        }),

    delete: (id: number) =>
        apiRequest<void>({ url: `/api/system/users/${id}`, method: "DELETE" }),

    getStatusOptions: () =>
        apiRequest<OptionItem[]>({ url: "/api/system/users/status-options" }),
};
