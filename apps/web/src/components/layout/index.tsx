import { LogoutOutlined, UserOutlined } from "@ant-design/icons";
import { ProLayout } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import { Link, useLocation, useRouter } from "@tanstack/react-router";
import { Alert, Button, Dropdown, Spin, type MenuProps } from "antd";
import { useMemo, type CSSProperties, type ReactNode, useState } from "react";

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

const layoutContentStyle: CSSProperties = {
    flex: 1,
    display: "flex",
    minHeight: 0,
    overflow: "hidden",
    padding: 0,
};

const layoutStyle: CSSProperties = {
    flex: 1,
    minHeight: 0,
    overflow: "hidden",
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

    const proMenuData = useMemo(() => convertToProRoutes(menuData), [menuData]);

    const handleSearchSelect = (path: AppRoutePath) => {
        void router.navigate({ to: path });
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

    return (
        <ProLayout
            className="app-shell flex min-h-0 flex-1 flex-col overflow-hidden"
            style={layoutStyle}
            title={APP_BRAND_NAME}
            logo="/rustzen.png"
            route={{ path: "/", routes: proMenuData }}
            location={{ pathname: currentPath }}
            layout="mix"
            siderWidth={232}
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
            contentStyle={layoutContentStyle}
            actionsRender={() => [
                <AppSearch key="page-search" routes={searchRoutes} onSelect={handleSearchSelect} />,
                <LanguageSwitch key="language" />,
                <ThemeSwitch key="theme" />,
            ]}
            avatarProps={{
                src: userInfo?.avatarUrl ?? undefined,
                size: "small",
                title: userInfo?.isSystem
                    ? localizeBuiltInUserName(userInfo.username, userInfo.realName)
                    : userInfo?.realName || userInfo?.username || t("账号", "Account"),
                render: (_props, dom) => (
                    <UserMenu userInfo={userInfo} onLogout={handleLogout}>
                        {dom}
                    </UserMenu>
                ),
            }}
        >
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

            <main className="min-h-0 w-full flex-1 overflow-x-hidden overflow-y-auto p-4 md:p-6">
                {children}
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
    children,
}: {
    userInfo: Auth.UserInfoResponse | null;
    onLogout: () => void;
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
                    }
                },
            }}
            trigger={["click"]}
        >
            <span className="inline-flex">{children}</span>
        </Dropdown>
    );
};
