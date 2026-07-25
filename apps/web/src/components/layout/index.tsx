import {
    MenuFoldOutlined,
    MenuUnfoldOutlined,
    SettingOutlined,
    UserOutlined,
} from "@ant-design/icons";
import { ProLayout } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import { Link, useLocation, useRouter } from "@tanstack/react-router";
import { Avatar, Button, Dropdown, type MenuProps } from "antd";
import { useMemo, type ReactNode, useState } from "react";

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
}

type ProRouteItem = {
    name: string;
    path?: string;
    icon?: ReactNode;
    routes?: ProRouteItem[];
    children?: ProRouteItem[];
    isMenu?: boolean;
    target?: "_blank" | "_self";
    disabled?: boolean;
};

export const BaseLayout = ({ children, hidden = false }: BaseLayoutProps) => {
    const [collapsed, setCollapsed] = useState(false);
    const userInfo = useAuthStore((state) => state.userInfo);
    const clearAuth = useAuthStore((state) => state.clearAuth);
    const checkMenuPermissions = useAuthStore((state) => state.checkMenuPermissions);
    const menuPermissionSignature = useAuthStore(
        (state) => state.userInfo?.permissions?.join("|") || "",
    );
    const locale = useLocale();
    const router = useRouter();
    const currentPath = useLocation().pathname;
    const { data: moduleNavigation = [] } = useQuery({
        queryKey: ["system", "modules", "navigation", menuPermissionSignature],
        queryFn: systemAPI.module.navigation,
        enabled: !hidden,
        refetchInterval: 10_000,
    });

    const menuData = useMemo(
        () => getMenuData(checkMenuPermissions, moduleNavigation),
        [checkMenuPermissions, menuPermissionSignature, moduleNavigation, locale],
    );

    const searchRoutes = useMemo(
        () => getSearchRouteItems(checkMenuPermissions, moduleNavigation),
        [checkMenuPermissions, menuPermissionSignature, moduleNavigation, locale],
    );

    const proMenuData = useMemo(() => convertToProRoutes(menuData), [menuData]);

    const pageTitle = useMemo(
        () =>
            currentPath === "/profile"
                ? t("个人资料", "Profile")
                : (currentPageTitle(menuData, currentPath) ?? APP_BRAND_NAME),
        [menuData, currentPath],
    );

    const handleSearchSelect = (path: AppRoutePath) => {
        void router.navigate({ to: path });
    };

    const handleLogout = async () => {
        await authAPI.logout();
        clearAuth();
        appMessage.success(t("退出登录成功", "Signed out successfully"));
        void router.navigate({ to: "/login" });
    };

    if (hidden) {
        return children;
    }

    return (
        <ProLayout
            className="rz-shell"
            title={APP_BRAND_NAME}
            logo={<img src="/rustzen.png" alt="" className="h-8 w-8 rounded-md" />}
            route={{ path: "/", routes: proMenuData }}
            location={{ pathname: currentPath }}
            collapsed={collapsed}
            onCollapse={setCollapsed}
            menu={{
                locale: false,
            }}
            menuItemRender={(item, dom) => {
                if (!item.path || item.children?.length) {
                    return <span>{dom}</span>;
                }
                return <Link to={item.path as AppRoutePath}>{dom}</Link>;
            }}
            contentStyle={{
                padding: 0,
            }}
        >
            <header className="rz-topbar flex h-14 shrink-0 items-center gap-3 border-b px-4">
                <Button
                    size="middle"
                    type="text"
                    icon={collapsed ? <MenuUnfoldOutlined /> : <MenuFoldOutlined />}
                    onClick={() => setCollapsed((value) => !value)}
                    aria-label={t("折叠导航", "Toggle sidebar")}
                />
                <div className="w-px border-r border-border/60" />
                <div className="min-w-0 flex-1">
                    <div className="truncate text-sm font-medium">{pageTitle}</div>
                </div>
                <AppSearch routes={searchRoutes} onSelect={handleSearchSelect} />
                <LanguageSwitch />
                <ThemeSwitch />
                <Button
                    type="text"
                    icon={<SettingOutlined />}
                    aria-label={t("偏好设置", "Preferences")}
                />
                <UserMenu userInfo={userInfo} onLogout={handleLogout} />
            </header>

            <main className="rz-content min-h-0 min-w-0 flex-1 overflow-hidden p-4 xl:p-5">
                <div className="rz-page mx-auto h-full min-h-0 w-full max-w-400 overflow-hidden">
                    {children}
                </div>
            </main>
        </ProLayout>
    );
};

const convertToProRoutes = (items: AppRouteItem[]): ProRouteItem[] =>
    items
        .map((item) => ({
            name: item.name,
            path: item.path,
            icon: item.icon,
            children: item.children?.length ? convertToProRoutes(item.children) : undefined,
            routes: item.children?.length ? convertToProRoutes(item.children) : undefined,
        }))
        .filter((item) => item.path || (item.children?.length ?? 0) > 0);

const UserMenu = ({
    userInfo,
    onLogout,
}: {
    userInfo: Auth.UserInfoResponse | null;
    onLogout: () => void;
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
            label: (
                <Link to="/profile" className="block w-full">
                    {t("个人资料", "Profile")}
                </Link>
            ),
        },
        {
            type: "divider",
        },
        {
            key: "logout",
            icon: <UserOutlined />,
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
                    }
                },
            }}
            trigger={["click"]}
        >
            <Button type="text" className="h-9 rounded-full p-0">
                <UserMenuAvatar userInfo={userInfo} />
                <span className="sr-only">{t("打开账号菜单", "Open account menu")}</span>
            </Button>
        </Dropdown>
    );
};

const UserMenuAvatar = ({ userInfo }: { userInfo: Auth.UserInfoResponse | null }) => {
    const displayName = userInfo?.realName || userInfo?.username || "RA";
    return (
        <Avatar size={32} src={userInfo?.avatarUrl ?? undefined} icon={<UserOutlined />}>
            {displayName.slice(0, 2).toUpperCase()}
        </Avatar>
    );
};

const currentPageTitle = (items: AppRouteItem[], currentPath: string): string | undefined => {
    for (const item of items) {
        if (item.path === currentPath) {
            return item.name;
        }
        if (item.children) {
            const childTitle = currentPageTitle(item.children, currentPath);
            if (childTitle) {
                return childTitle;
            }
        }
    }
    return undefined;
};
