import { create } from "zustand";
import { persist } from "zustand/middleware";

interface AuthState {
    userInfo: Auth.UserInfoResponse | null;
    token: string | null;
    authGeneration: number;
    handleLogin: (token: string, userInfo: Auth.UserInfoResponse) => void;
    updateToken: (params: string) => void;
    updateAvatar: (avatarUrl: string) => void;
    updateUserInfo: (params: Auth.UserInfoResponse) => void;
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
            handleLogin: (token, userInfo) => {
                set((state) => ({ token, userInfo, authGeneration: state.authGeneration + 1 }));
            },
            updateToken: (params: string) => {
                set((state) => ({ token: params, authGeneration: state.authGeneration + 1 }));
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
                set((state) => ({
                    userInfo: params,
                    authGeneration: sameAuthority(state.userInfo, params)
                        ? state.authGeneration
                        : state.authGeneration + 1,
                }));
            },
            clearAuth: () => {
                set((state) => ({
                    userInfo: null,
                    token: null,
                    authGeneration: state.authGeneration + 1,
                }));
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
        "/monitoring/incidents": "monitor:incident:view",
        "/monitoring/summaries": "monitor:node:view",
        "/analytics": "insights:overview:view",
        "/analytics/overview": "insights:overview:view",
        "/analytics/details": "insights:event:view",
        "/reports": "reports:flow:view",
        "/reports/templates": ["reports:flow:view", "reports:schedule:view"],
        "/reports/runs": "reports:run:view",
        "/system/module-log": "system:module:log:view",
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
