import { create } from "zustand";
import { persist } from "zustand/middleware";

interface AuthState {
    userInfo: Auth.UserInfoResponse | null;
    token: string | null;
    handleLogin: (token: string, userInfo: Auth.UserInfoResponse) => void;
    updateToken: (params: string) => void;
    updateAvatar: (avatarUrl: string) => void;
    updateUserInfo: (params: Auth.UserInfoResponse) => void;
    clearAuth: () => void;
    checkPermissions: (code: string) => boolean;
    checkMenuPermissions: (path: string) => boolean;
}

export const useAuthStore = create<AuthState>()(
    persist(
        (set, get) => ({
            userInfo: null,
            token: null,
            handleLogin: (token, userInfo) => {
                set({ token, userInfo });
            },
            updateToken: (params: string) => {
                set({ token: params });
            },
            updateAvatar: (avatarUrl: string) => {
                set({
                    userInfo: {
                        ...(get().userInfo as Auth.UserInfoResponse),
                        avatarUrl,
                    },
                });
            },
            updateUserInfo: (params: Auth.UserInfoResponse) => {
                set({ userInfo: params });
            },
            clearAuth: () => {
                set({ userInfo: null, token: null });
            },
            checkPermissions: (code: string) => {
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
                const codeArr = code.split(":");
                for (let i = codeArr.length - 1; i > 0; i--) {
                    const prefix = codeArr.slice(0, i).join(":") + ":*";
                    if (permissions.includes(prefix)) {
                        return true;
                    }
                }
                return false;
            },
            checkMenuPermissions: (path: string) => {
                return getRouteCapabilityCodes(path).some((code) => get().checkPermissions(code));
            },
        }),
        {
            name: "auth-store",
        },
    ),
);

export const getRouteCapabilityCodes = (pathname: string): string[] => {
    const explicitRouteCapability: Record<string, string | string[]> = {
        "/": "dashboard:view",
        "/monitoring": "monitor:overview:view",
        "/monitoring/overview": "monitor:overview:view",
        "/monitoring/nodes": "monitor:node:view",
        "/monitoring/checks": "monitor:check:view",
        "/monitoring/incidents": "monitor:incident:view",
        "/analytics": "insights:overview:view",
        "/analytics/overview": "insights:overview:view",
        "/analytics/details": "insights:event:view",
        "/reports": "reports:flow:view",
        "/reports/templates": ["reports:flow:view", "reports:schedule:view"],
        "/reports/runs": "reports:run:view",
    };
    const explicitCode = explicitRouteCapability[pathname];
    if (Array.isArray(explicitCode)) {
        return explicitCode;
    }
    if (explicitCode) {
        return [explicitCode];
    }

    const code = pathname.replace(/\//g, ":").slice(1);
    if (code.endsWith(":create")) {
        return [code];
    }
    if (code.endsWith(":edit") || code.endsWith(":detail")) {
        return [
            code
                .split(":")
                .filter((s) => !/^\d+$/.test(s))
                .join(":"),
        ];
    }
    return [`${code}:list`];
};
