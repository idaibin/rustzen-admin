import { useQuery } from "@tanstack/react-query";
import { createRootRoute, Navigate, Outlet, redirect } from "@tanstack/react-router";
import { lazy, Suspense, useEffect } from "react";

import { MessageContent, authAPI } from "@/api";
import { BaseLayout } from "@/components/layout";
import { ThemeProvider } from "@/components/theme-provider";
import { useLocale } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

import { NotificationShell } from "./-notifications-shell";

const permissionFreePaths = new Set(["/profile", "/403", "/404"]);

const ReactQueryDevtools = import.meta.env.DEV
    ? lazy(() =>
          import("@tanstack/react-query-devtools").then((module) => ({
              default: module.ReactQueryDevtools,
          })),
      )
    : null;

const TanStackRouterDevtools = import.meta.env.DEV
    ? lazy(() =>
          import("@tanstack/react-router-devtools").then((module) => ({
              default: module.TanStackRouterDevtools,
          })),
      )
    : null;

export const Route = createRootRoute({
    beforeLoad: (ctx: { location: { pathname: string } }) => {
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
            throw redirect({ to: "/" });
        }

        // Redirect to home skip permissions check
        if (permissionFreePaths.has(curPath)) {
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
    useLocale();
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
        <ThemeProvider>
            <BaseLayout hidden={!token} headerActions={token ? <NotificationShell /> : null}>
                <Outlet />
            </BaseLayout>
            <MessageContent />
            {ReactQueryDevtools && TanStackRouterDevtools && (
                <Suspense fallback={null}>
                    <ReactQueryDevtools buttonPosition="bottom-right" />
                    <TanStackRouterDevtools />
                </Suspense>
            )}
        </ThemeProvider>
    );
}
