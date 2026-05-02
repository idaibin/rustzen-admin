import { apiRequest } from "@/api/request";

export const accountAPI = {
    updateProfile: (data: Account.UpdateProfileRequest) => {
        return apiRequest<Auth.UserInfoResponse, Account.UpdateProfileRequest>({
            url: "/api/account/profile",
            method: "PUT",
            params: data,
        });
    },

    changePassword: (data: Account.ChangePasswordRequest) => {
        return apiRequest<void, Account.ChangePasswordRequest>({
            url: "/api/account/password",
            method: "PUT",
            params: data,
        });
    },
};
