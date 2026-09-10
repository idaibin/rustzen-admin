import { create } from "zustand";
import { persist } from "zustand/middleware";

interface AuthState {
    userInfo: Auth.UserInfoResponse | null;
    token: string | null;
    authGeneration: number;
    handleLogin: (token: string, userInfo: Auth.UserInfoResponse) => void;
    updateToken: (token: string) => void;
    updateAvatar: (avatarUrl: string) => void;
    updateUserInfo: (userInfo: Auth.UserInfoResponse) => void;
    clearAuth: () => void;
    checkPermissions: (code: string) => boolean;
    checkMenuPermissions: (path: string) => boolean;
}

const sameAuthority = (left: Auth.UserInfoResponse | null, right: Auth.UserInfoResponse) =>
    left?.id === right.id &&
    left.username === right.username &&
    left.isSystem === right.isSystem &&
    JSON.stringify([...(left.permissions ?? [])].sort()) ===
        JSON.stringify([...(right.permissions ?? [])].sort());

export const useAuthStore = create<AuthState>()(
    persist(
        (set, get) => ({
            userInfo: null,
            token: null,
            authGeneration: 0,
            handleLogin: (token, userInfo) =>
                set((state) => ({ token, userInfo, authGeneration: state.authGeneration + 1 })),
            updateToken: (token) =>
                set((state) => ({ token, authGeneration: state.authGeneration + 1 })),
            updateAvatar: (avatarUrl) =>
                set({ userInfo: { ...(get().userInfo as Auth.UserInfoResponse), avatarUrl } }),
            updateUserInfo: (userInfo) =>
                set((state) => ({
                    userInfo,
                    authGeneration: sameAuthority(state.userInfo, userInfo)
                        ? state.authGeneration
                        : state.authGeneration + 1,
                })),
            clearAuth: () =>
                set((state) => ({
                    userInfo: null,
                    token: null,
                    authGeneration: state.authGeneration + 1,
                })),
            checkPermissions: (code) => {
                const permissions = get().userInfo?.permissions || [];
                if (permissions.includes("*") || permissions.includes(code)) return true;
                const parts = code.split(":");
                return parts
                    .slice(0, -1)
                    .some((_, index) =>
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
        "/": "insights:overview:view",
        "/analytics": "insights:overview:view",
        "/analytics/overview": "insights:overview:view",
        "/analytics/details": "insights:event:view",
        "/system/user": "system:user:list",
        "/system/role": "system:role:list",
    };
    return routes[pathname] ? [routes[pathname]] : [];
};
