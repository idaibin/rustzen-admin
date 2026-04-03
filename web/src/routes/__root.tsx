import { useQuery } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import { createRootRoute, Navigate, Outlet, redirect } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import { App, ConfigProvider } from "antd";
import { useEffect } from "react";

import { MessageContent, authAPI } from "@/api";
import { BaseLayout } from "@/components/base-layout";
import { useAuthStore } from "@/store/useAuthStore";

export const Route = createRootRoute({
    beforeLoad: (ctx) => {
        const curPath = ctx.location.pathname;
        const { token, checkMenuPermissions } = useAuthStore.getState();

        // Redirect to login if no token
        if (!token) {
            if (curPath !== "/login") {
                throw redirect({ to: "/login" });
            }
            return null;
        }

        // Redirect to home if already logged in
        if (curPath === "/login") {
            console.log("Redirect to home");
            throw redirect({ to: "/" });
        }

        // Redirect to home skip permissions check
        if (curPath === "/") {
            return null;
        }
        const isPermission = checkMenuPermissions(curPath);

        // Redirect to 403 if no permission
        if (token && !isPermission) {
            throw redirect({ to: "/403" });
        }
    },
    component: RootLayout,
    notFoundComponent: () => <Navigate to="/404" />,
});

function RootLayout() {
    const { token, updateUserInfo } = useAuthStore();
    const { data: userInfo } = useQuery({
        queryKey: ["auth", "me"],
        queryFn: authAPI.me,
        enabled: !!token,
    });

    useEffect(() => {
        if (userInfo) {
            updateUserInfo(userInfo);
        }
    }, [userInfo, updateUserInfo]);

    return (
        <ConfigProvider>
            <App>
                <BaseLayout hidden={!token}>
                    <Outlet />
                </BaseLayout>
                <MessageContent />
            </App>
            <ReactQueryDevtools buttonPosition="bottom-right" />
            <TanStackRouterDevtools />
        </ConfigProvider>
    );
}
