import { apiRequest, proTableRequest } from "@/api";

/**
 * 用户管理API服务
 */
export const userAPI = {
    listUsers: (params?: User.QueryParams) =>
        proTableRequest<User.Item, User.QueryParams>({
            url: "/api/system/users",
            params,
        }),

    createUser: (data: User.CreateRequest) =>
        apiRequest<User.Item, User.CreateRequest>({
            url: "/api/system/users",
            method: "POST",
            params: data,
        }),

    updateUser: (id: number, data: User.UpdateRequest) =>
        apiRequest<User.Item, User.UpdateRequest>({
            url: `/api/system/users/${id}`,
            method: "PUT",
            params: data,
        }),

    deleteUser: (id: number) => apiRequest<void>({ url: `/api/system/users/${id}`, method: "DELETE" }),

    updateUserStatus: (id: number, status: number) =>
        apiRequest<void>({
            url: `/api/system/users/${id}/status`,
            method: "PUT",
            params: { status },
        }),

    updateUserPassword: (id: number, password: string) =>
        apiRequest<void>({
            url: `/api/system/users/${id}/reset-password`,
            method: "PUT",
            params: { password },
        }),

    listUserStatusOptions: () =>
        apiRequest<Api.OptionItem<number>[]>({
            url: "/api/system/users/status-options",
        }),
};
