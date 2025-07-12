import { request } from "../api";
import type { LoginRequest, LoginResponse, UserInfoResponse } from "Auth";

/**
 * 认证相关API服务
 */
export const authAPI = {
    /**
     * 用户登录
     */
    login: (data: LoginRequest) =>
        request.post<LoginResponse, LoginRequest>("/api/auth/login", data, {
            successMessage: "Login successful",
        }),

    /**
     * 用户登出
     */
    logout: () => request.get("/api/auth/logout"),

    /**
     * 获取当前用户信息
     */
    getUserInfo: () => request.get<UserInfoResponse>("/api/auth/me"),

    /**
     * 检查用户是否已登录
     */
    isLoggedIn: () => !!localStorage.getItem("token"),

    /**
     * 获取存储的JWT token
     */
    getToken: (): string | null => {
        return localStorage.getItem("token");
    },

    /**
     * 获取缓存的用户信息
     */
    getCachedUserInfo: () => {
        const userInfo = localStorage.getItem("userInfo");
        try {
            return userInfo ? JSON.parse(userInfo) : null;
        } catch (error) {
            console.error("Failed to parse cached user info:", error);
            localStorage.removeItem("userInfo");
            return null;
        }
    },

    // URL生成器（SWR使用）
    urls: {
        login: () => "/api/auth/login",
        logout: () => "/api/auth/logout",
        getUserInfo: () => "/api/auth/me",
    },

    // Token管理
    saveToken: (token: string) => {
        localStorage.setItem("token", token);
    },

    removeToken: () => {
        localStorage.removeItem("token");
        localStorage.removeItem("userInfo");
    },

    isTokenValid: (): boolean => {
        const token = localStorage.getItem("token");
        return !!token;
    },
};
