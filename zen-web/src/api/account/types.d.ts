declare namespace Account {
    interface UpdateProfileRequest {
        email: string;
        realName?: string;
    }

    interface ChangePasswordRequest {
        currentPassword: string;
        newPassword: string;
        confirmPassword: string;
    }
}
