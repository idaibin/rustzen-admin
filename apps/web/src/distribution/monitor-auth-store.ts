import { create } from "zustand";
import { persist } from "zustand/middleware";

interface AuthState {
    userInfo: Auth.UserInfoResponse | null;
    token: string | null;
    handleLogin: (token: string, userInfo: Auth.UserInfoResponse) => void;
    updateToken: (token: string) => void;
    updateAvatar: (avatarUrl: string) => void;
    updateUserInfo: (userInfo: Auth.UserInfoResponse) => void;
    clearAuth: () => void;
    checkPermissions: (code: string) => boolean;
    checkMenuPermissions: (path: string) => boolean;
}

export const useAuthStore = create<AuthState>()(
    persist(
        (set, get) => ({
            userInfo: null,
            token: null,
            handleLogin: (token, userInfo) => set({ token, userInfo }),
            updateToken: (token) => set({ token }),
            updateAvatar: (avatarUrl) =>
                set({ userInfo: { ...(get().userInfo as Auth.UserInfoResponse), avatarUrl } }),
            updateUserInfo: (userInfo) => set({ userInfo }),
            clearAuth: () => set({ userInfo: null, token: null }),
            checkPermissions: (code) => {
                const permissions = get().userInfo?.permissions || [];
                if (permissions.includes("*") || permissions.includes(code)) return true;
                const parts = code.split(":");
                return parts.slice(0, -1).some((_, index) =>
                    permissions.includes(`${parts.slice(0, index + 1).join(":")}:*`),
                );
            },
            checkMenuPermissions: (path) =>
                getRouteCapabilityCodes(path).some((code) => get().checkPermissions(code)),
        }),
        { name: "auth-store" },
    ),
);

export const getRouteCapabilityCodes = (pathname: string): string[] => {
    const routes: Record<string, string> = {
        "/monitoring": "monitor:overview:view",
        "/monitoring/overview": "monitor:overview:view",
        "/monitoring/nodes": "monitor:node:view",
        "/monitoring/incidents": "monitor:incident:view",
        "/monitoring/summaries": "monitor:node:view",
        "/system/user": "system:user:list",
        "/system/role": "system:role:list",
    };
    return routes[pathname] ? [routes[pathname]] : [];
};
