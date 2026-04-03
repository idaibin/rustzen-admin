import { apiRequest } from "@/api/request";

/**
 * User management API service.
 */
export const userAPI = {
    list: async (params: User.QueryParams) => {
        const res = await apiRequest({
            url: "/api/system/users",
            params,
            raw: true,
        });
        return {
            data: res.data as User.Item[],
            total: res.total ?? 0,
            success: true,
        };
    },
    create: (data: User.CreateRequest) => {
        return apiRequest<User.Item, User.CreateRequest>({
            url: "/api/system/users",
            method: "POST",
            params: data,
        });
    },
    update: (id: number, data: User.UpdateRequest) => {
        return apiRequest<User.Item, User.UpdateRequest>({
            url: `/api/system/users/${id}`,
            method: "PUT",
            params: data,
        });
    },
    delete: (id: number) => {
        return apiRequest<void>({
            url: `/api/system/users/${id}`,
            method: "DELETE",
        });
    },
    status: (id: number, status: number) => {
        return apiRequest<void>({
            url: `/api/system/users/${id}/status`,
            method: "PUT",
            params: { status },
        });
    },
    password: (id: number, password: string) => {
        return apiRequest<void>({
            url: `/api/system/users/${id}/reset-password`,
            method: "PUT",
            params: { password },
        });
    },
    statusOptions: () => {
        return apiRequest<Api.OptionItem<number>[]>({
            url: "/api/system/users/status-options",
        });
    },
};
