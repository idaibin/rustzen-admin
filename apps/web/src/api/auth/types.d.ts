declare namespace Auth {
    interface LoginRequest {
        username: string;
        password: string;
    }

    interface LoginResponse {
        token: string;
        userInfo: UserInfoResponse;
    }

    interface UserInfoResponse {
        id: number;
        username: string;
        email?: string;
        realName?: string;
        avatarUrl?: string;
        permissions: string[];
        isSystem: boolean;
    }

}
