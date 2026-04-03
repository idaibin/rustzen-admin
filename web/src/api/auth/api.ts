import { apiRequest } from "@/api/request";

/**
 * Authentication API service.
 */
export const authAPI = {
    login: (data: Auth.LoginRequest) => {
        return apiRequest<Auth.LoginResponse, Auth.LoginRequest>({
            url: "/api/auth/login",
            method: "POST",
            params: data,
        });
    },

    logout: () => {
        return apiRequest<void>({ url: "/api/auth/logout" });
    },

    me: () => {
        return apiRequest<Auth.UserInfoResponse>({ url: "/api/auth/me" });
    },
};
