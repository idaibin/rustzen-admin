import { apiRequest } from "../request";
import type { LoginRequest, LoginResponse, UserInfoResponse } from "Auth";

/**
 * 认证相关API服务
 */
export const authAPI = {
    /**
     * 用户登录
     */
    login: (data: LoginRequest) =>
        apiRequest<LoginResponse, LoginRequest>({
            url: "/api/auth/login",
            method: "POST",
            params: data,
        }),

    /**
     * 用户登出
     */
    logout: () => apiRequest<void>({ url: "/api/auth/logout" }),

    /**
     * 获取当前用户信息
     */
    getUserInfo: () => apiRequest<UserInfoResponse>({ url: "/api/auth/me" }),
};
