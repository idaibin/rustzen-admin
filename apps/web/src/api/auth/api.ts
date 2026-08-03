import { apiRequest } from "@/api/request";
import { getCurrentAdminUser } from "@/api/generated/admin-contract";

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

    me: async () => (await getCurrentAdminUser()).data,
};
