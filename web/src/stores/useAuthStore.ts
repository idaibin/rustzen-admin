import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { LoginResponse, UserInfoResponse } from "Auth";

interface AuthState {
    userInfo: UserInfoResponse | null;
    token: string | null;
    updateUserInfo: (params: UserInfoResponse) => void;
    updateToken: (params: string) => void;
    setAuth: (params: LoginResponse) => void;
    clearAuth: () => void;
    checkPermision: (code: string, isPage?: boolean) => boolean;
}

export const useAuthStore = create<AuthState>()(
    persist(
        (set, get) => ({
            userInfo: null,
            token: null,
            updateUserInfo: (params: UserInfoResponse) => {
                set({ userInfo: params });
            },
            updateToken: (params: string) => {
                set({ token: params });
            },
            setAuth: (params: LoginResponse) => {
                set({ userInfo: params.userInfo, token: params.token });
            },
            // Clear all auth state
            clearAuth: () => {
                console.log("clearAuth");
                set({
                    userInfo: null,
                    token: null,
                });
            },
            checkPermision: (code: string, isPage) => {
                const permissions = get().userInfo?.permissions || [];
                if (permissions.length === 0) {
                    return false;
                }
                if (permissions.includes("*")) {
                    return true;
                }
                if (permissions.includes(code)) {
                    return true;
                }
                // 逐级判断 system:user:list -> system:user:*, system:*
                const codeArr = code.split(":");
                for (let i = codeArr.length - 1; i > 0; i--) {
                    const prefix = codeArr.slice(0, i).join(":") + ":*";
                    if (permissions.includes(prefix)) {
                        return true;
                    }
                }

                if (isPage) {
                    const hasPage = permissions.some(
                        (p) => p.endsWith(":list") && p.startsWith(code)
                    );
                    if (hasPage) {
                        return true;
                    }
                }
                return false;
            },
        }),
        {
            name: "auth-store",
        }
    )
);
