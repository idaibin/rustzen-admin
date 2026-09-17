import { MenuOutlined, LogoutOutlined, UserOutlined } from "@ant-design/icons";
import { useQuery } from "@tanstack/react-query";
import { Link, useLocation, useRouter } from "@tanstack/react-router";
import { Alert, Avatar, Button, Drawer, Dropdown, Grid, Menu, Spin, type MenuProps } from "antd";
import { useEffect, useMemo, type ReactNode, useState } from "react";

import { appMessage, authAPI, systemAPI } from "@/api";
import { LanguageSwitch } from "@/components/language-switch";
import { ThemeSwitch } from "@/components/theme-provider";
import { APP_BRAND_NAME } from "@/constant/brand";
import { localizeBuiltInUserName } from "@/lib/builtin-i18n";
import { t, useLocale } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

import { AppSearch } from "./app-search";
import { getMenuData, getSearchRouteItems, type AppRouteItem, type AppRoutePath } from "./routes";

interface BaseLayoutProps {
    children: ReactNode;
    hidden?: boolean;
    headerActions?: ReactNode;
}

export const BaseLayout = ({ children, hidden = false, headerActions }: BaseLayoutProps) => {
    const [mobileOpen, setMobileOpen] = useState(false);
    const [openKeys, setOpenKeys] = useState<string[]>([]);
    const screens = Grid.useBreakpoint();
    const userInfo = useAuthStore((state) => state.userInfo);
    const clearAuth = useAuthStore((state) => state.clearAuth);
    const checkMenuPermissions = useAuthStore((state) => state.checkMenuPermissions);
    const menuPermissionSignature = useAuthStore(
        (state) => state.userInfo?.permissions?.join("|") || "",
    );
    const locale = useLocale();
    const router = useRouter();
    const currentPath = useLocation().pathname;
    const {
        data: moduleNavigation,
        error: moduleNavigationError,
        isPending: isModuleNavigationPending,
        refetch: refetchModuleNavigation,
    } = useQuery({
        queryKey: ["system", "modules", "navigation", menuPermissionSignature],
        queryFn: systemAPI.module.navigation,
        enabled: !hidden,
        refetchInterval: 10_000,
    });

    const menuData = useMemo(
        () => getMenuData(checkMenuPermissions, moduleNavigation ?? []),
        [checkMenuPermissions, menuPermissionSignature, moduleNavigation, locale],
    );

    const searchRoutes = useMemo(
        () => getSearchRouteItems(checkMenuPermissions, moduleNavigation ?? []),
        [checkMenuPermissions, menuPermissionSignature, moduleNavigation, locale],
    );

    const menuItems = useMemo(() => toMenuItems(menuData), [menuData]);
    useEffect(() => {
        const parent = menuData.find((item) =>
            item.children?.some((child) => child.path === currentPath),
        );
        const parentPath = parent?.path;
        if (parentPath)
            setOpenKeys((keys) => (keys.includes(parentPath) ? keys : [...keys, parentPath]));
        setMobileOpen(false);
    }, [currentPath, menuData]);

    const handleSearchSelect = (path: AppRoutePath) => {
        void router.navigate({ to: path });
    };
    const handleNavigationSelect = (key: string) => {
        setMobileOpen(false);
        if (key.startsWith("/")) void router.navigate({ to: key as AppRoutePath });
    };

    const handleLogout = async () => {
        try {
            await authAPI.logout();
            appMessage.success(t("退出登录成功", "Signed out successfully"));
        } finally {
            clearAuth();
            void router.navigate({ to: "/login" });
        }
    };

    if (hidden) {
        return children;
    }

    const navigation = (
        <Menu
            aria-label={t("主导航", "Main navigation")}
            mode="inline"
            inlineCollapsed={false}
            selectedKeys={[currentPath]}
            openKeys={openKeys}
            onOpenChange={setOpenKeys}
            onClick={({ key }) => handleNavigationSelect(String(key))}
            items={menuItems}
            style={{ width: "100%", borderInlineEnd: 0, background: "transparent" }}
        />
    );
    const accountName = userInfo?.isSystem
        ? localizeBuiltInUserName(userInfo.username, userInfo.realName)
        : userInfo?.realName || userInfo?.username || t("账号", "Account");

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
                <nav className="shell-navigation">{screens.md ? navigation : null}</nav>
            </aside>
            <header className="shell-header">
                <Button
                    className="shell-mobile-toggle"
                    type="text"
                    icon={<MenuOutlined />}
                    aria-label={t("打开导航", "Open navigation")}
                    onClick={() => setMobileOpen(true)}
                />
                <div className="shell-search">
                    <AppSearch routes={searchRoutes} onSelect={handleSearchSelect} />
                </div>
                <div className="flex shrink-0 items-center gap-2">
                    {headerActions}
                    <LanguageSwitch />
                    <ThemeSwitch />
                    <UserMenu
                        userInfo={userInfo}
                        onLogout={handleLogout}
                        onNavigate={handleNavigationSelect}
                    >
                        <Button
                            type="text"
                            aria-label={t("账号菜单", "Account menu")}
                            className="shell-account"
                        >
                            <Avatar
                                size="small"
                                src={userInfo?.avatarUrl ?? undefined}
                                icon={<UserOutlined />}
                            />
                            <span className="hidden sm:inline">{accountName}</span>
                        </Button>
                    </UserMenu>
                </div>
            </header>
            <main className="shell-content">
                <div className="shell-page">
                    {isModuleNavigationPending ? (
                        <Spin
                            description={t("正在加载模块导航…", "Loading module navigation…")}
                            size="small"
                        />
                    ) : null}
                    {moduleNavigationError ? (
                        <Alert
                            type="warning"
                            showIcon
                            message={t(
                                "模块导航加载失败，请重试。",
                                "Module navigation could not be loaded. Please retry.",
                            )}
                            action={
                                <Button size="small" onClick={() => void refetchModuleNavigation()}>
                                    {t("重试", "Retry")}
                                </Button>
                            }
                        />
                    ) : null}
                    {children}
                </div>
            </main>
            <Drawer
                title={APP_BRAND_NAME}
                placement="left"
                open={mobileOpen && !screens.md}
                onClose={() => setMobileOpen(false)}
                size={280}
                styles={{ body: { padding: 8, background: "var(--sidebar)" } }}
            >
                {!screens.md ? navigation : null}
            </Drawer>
        </div>
    );
};

const toMenuItems = (items: AppRouteItem[]): MenuProps["items"] =>
    items.map((item) => ({
        key: item.path || item.name,
        icon: item.icon,
        label:
            item.permission === "reports:schedule:view" ? (
                <span data-testid="navigation-reports-schedules" data-label={item.name}>
                    {item.name}
                </span>
            ) : (
                item.name
            ),
        children: item.children?.length ? toMenuItems(item.children) : undefined,
    }));

const UserMenu = ({
    userInfo,
    onLogout,
    onNavigate,
    children,
}: {
    userInfo: Auth.UserInfoResponse | null;
    onLogout: () => void;
    onNavigate: (path: AppRoutePath) => void;
    children: ReactNode;
}) => {
    const menuItems: MenuProps["items"] = [
        {
            key: "username",
            label: (
                <div className="max-w-44">
                    <div className="truncate text-sm font-medium">
                        {userInfo?.isSystem
                            ? localizeBuiltInUserName(userInfo.username, userInfo.realName)
                            : userInfo?.realName || userInfo?.username || t("账号", "Account")}
                    </div>
                    <div className="truncate text-xs text-muted-foreground">
                        {userInfo?.username || t("个人资料", "Profile")}
                    </div>
                </div>
            ),
            disabled: true,
        },
        {
            type: "divider",
        },
        {
            key: "profile",
            icon: <UserOutlined />,
            label: t("个人资料", "Profile"),
        },
        {
            type: "divider",
        },
        {
            key: "logout",
            icon: <LogoutOutlined />,
            danger: true,
            label: t("退出登录", "Sign out"),
        },
    ];

    return (
        <Dropdown
            menu={{
                items: menuItems,
                onClick: ({ key }) => {
                    if (key === "logout") {
                        onLogout();
                    } else if (key === "profile") {
                        onNavigate("/profile");
                    }
                },
            }}
            trigger={["click"]}
        >
            <span className="inline-flex">{children}</span>
        </Dropdown>
    );
};
