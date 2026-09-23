import { installationAPI } from "@/api/installation/api";
import { apiRequest } from "@/api/request";
import { appMessage, MessageContent } from "@/api/runtime";

const authAPI = {
    login: (data: Auth.LoginRequest) =>
        apiRequest<Auth.LoginResponse, Auth.LoginRequest>({
            url: "/api/auth/login",
            method: "POST",
            params: data,
        }),
    logout: () => apiRequest<void>({ url: "/api/auth/logout" }),
    me: () => apiRequest<Auth.UserInfoResponse>({ url: "/api/auth/me" }),
};

const accountAPI = {
    updateProfile: (data: Account.UpdateProfileRequest) =>
        apiRequest<Auth.UserInfoResponse, Account.UpdateProfileRequest>({
            url: "/api/account/profile",
            method: "PUT",
            params: data,
        }),
    changePassword: (data: Account.ChangePasswordRequest) =>
        apiRequest<void, Account.ChangePasswordRequest>({
            url: "/api/account/password",
            method: "PUT",
            params: data,
        }),
};

const systemAPI = {
    user: {
        list: async (params: User.QueryParams) => {
            const result = await apiRequest<User.Item[], User.QueryParams>({
                url: "/api/system/users",
                params,
                raw: true,
            });
            return { data: result.data, total: result.total ?? 0, success: true };
        },
        create: (data: User.CreateRequest) =>
            apiRequest<void, User.CreateRequest>({
                url: "/api/system/users",
                method: "POST",
                params: data,
            }),
        update: (id: number, data: User.UpdateRequest) =>
            apiRequest<number, User.UpdateRequest>({
                url: `/api/system/users/${id}`,
                method: "PUT",
                params: data,
            }),
        delete: (id: number) =>
            apiRequest<void>({ url: `/api/system/users/${id}`, method: "DELETE" }),
        status: (id: number, status: number) =>
            apiRequest<boolean>({
                url: `/api/system/users/${id}/status`,
                method: "PUT",
                params: { status },
            }),
        password: (id: number, password: string) =>
            apiRequest<boolean>({
                url: `/api/system/users/${id}/password`,
                method: "PUT",
                params: { password },
            }),
    },
    role: {
        list: async (params: Role.QueryParams) => {
            const result = await apiRequest<Role.Item[], Role.QueryParams>({
                url: "/api/system/roles",
                params,
                raw: true,
            });
            return { data: result.data, total: result.total ?? 0, success: true };
        },
        create: (data: Role.CreateRequest) =>
            apiRequest<void, Role.CreateRequest>({
                url: "/api/system/roles",
                method: "POST",
                params: data,
            }),
        update: (id: number, data: Role.UpdateRequest) =>
            apiRequest<void, Role.UpdateRequest>({
                url: `/api/system/roles/${id}`,
                method: "PUT",
                params: data,
            }),
        delete: (id: number) =>
            apiRequest<void>({ url: `/api/system/roles/${id}`, method: "DELETE" }),
        options: () => apiRequest<Role.OptionItem[]>({ url: "/api/system/roles/options" }),
    },
};

export { accountAPI, appMessage, authAPI, installationAPI, MessageContent, systemAPI };
