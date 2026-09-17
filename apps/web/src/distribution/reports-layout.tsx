import { LogoutOutlined, FileTextOutlined, TeamOutlined, UserOutlined } from "@ant-design/icons";
import { Link, useLocation, useRouter } from "@tanstack/react-router";
import { Avatar, Button, Dropdown, Menu, type MenuProps } from "antd";
import type { ReactNode } from "react";

import { appMessage, authAPI } from "@/api";
import { LanguageSwitch } from "@/components/language-switch";
import { ThemeSwitch } from "@/components/theme-provider";
import { APP_BRAND_NAME } from "@/constant/brand";
import { t } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

const routes = [
    ["/reports/templates", "模板", "Templates", "reports:flow:view", <FileTextOutlined />],
    ["/reports/runs", "填报执行", "Runs", "reports:run:view", <FileTextOutlined />],
    ["/system/user", "用户", "Users", "system:user:list", <UserOutlined />],
    ["/system/role", "角色", "Roles", "system:role:list", <TeamOutlined />],
] as const;

export const BaseLayout = ({
    children,
    hidden = false,
    headerActions,
}: {
    children: ReactNode;
    hidden?: boolean;
    headerActions?: ReactNode;
}) => {
    const location = useLocation();
    const router = useRouter();
    const userInfo = useAuthStore((state) => state.userInfo);
    const clearAuth = useAuthStore((state) => state.clearAuth);
    const checkPermissions = useAuthStore((state) => state.checkPermissions);
    if (hidden) return children;
    const menuItems: MenuProps["items"] = routes
        .filter(([, , , permission]) => checkPermissions(permission))
        .map(([path, chinese, english, , icon]) => ({
            key: path,
            icon,
            label: t(chinese, english),
        }));
    const logout = async () => {
        try {
            await authAPI.logout();
            appMessage.success(t("退出登录成功", "Signed out successfully"));
        } finally {
            clearAuth();
            void router.navigate({ to: "/login" });
        }
    };
    return (
        <div className="app-shell">
            <aside className="shell-sidebar" aria-label={t("侧栏", "Sidebar")}>
                <div className="shell-brand">
                    <Link
                        to="/"
                        className="flex min-w-0 flex-1 items-center gap-2 text-foreground no-underline"
                    >
                        <img src="/rustzen.png" alt="" className="size-7 shrink-0" />
                        <span className="truncate text-base font-semibold">{APP_BRAND_NAME}</span>
                    </Link>
                </div>
                <nav className="shell-navigation">
                    <Menu
                        mode="inline"
                        selectedKeys={[location.pathname]}
                        items={menuItems}
                        onClick={({ key }) =>
                            void router.navigate({ to: key as "/reports/templates" })
                        }
                        className="w-full !border-e-0"
                    />
                </nav>
            </aside>
            <header className="shell-header">
                <div className="ms-auto flex shrink-0 items-center gap-2">
                    {headerActions}
                    <LanguageSwitch />
                    <ThemeSwitch />
                    <Dropdown
                        trigger={["click"]}
                        menu={{
                            items: [
                                {
                                    key: "profile",
                                    icon: <UserOutlined />,
                                    label: t("个人资料", "Profile"),
                                },
                                { type: "divider" },
                                {
                                    key: "logout",
                                    icon: <LogoutOutlined />,
                                    danger: true,
                                    label: t("退出登录", "Sign out"),
                                },
                            ],
                            onClick: ({ key }) =>
                                key === "logout"
                                    ? void logout()
                                    : void router.navigate({ to: "/profile" }),
                        }}
                    >
                        <Button type="text" aria-label={t("账号菜单", "Account menu")}>
                            <Avatar
                                size="small"
                                src={userInfo?.avatarUrl ?? undefined}
                                icon={<UserOutlined />}
                            />
                            <span className="hidden sm:inline">
                                {userInfo?.realName || userInfo?.username || t("账号", "Account")}
                            </span>
                        </Button>
                    </Dropdown>
                </div>
            </header>
            <main className="shell-content">
                <div className="shell-page">{children}</div>
            </main>
        </div>
    );
};
