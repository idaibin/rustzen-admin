import { useMemo, type CSSProperties } from "react";
import {
    BookOutlined,
    DashboardOutlined,
    FileUnknownOutlined,
    HistoryOutlined,
    LogoutOutlined,
    MenuOutlined,
    SettingOutlined,
    StopOutlined,
    TeamOutlined,
    UserOutlined,
} from "@ant-design/icons";
import { ProLayout } from "@ant-design/pro-components";
import { Link, useLocation, useRouter } from "@tanstack/react-router";
import type { MenuProps } from "antd";
import { Dropdown } from "antd";

import { appMessage, authAPI } from "@/api";
import { useAuthStore } from "@/store/useAuthStore";

type AppRouter = {
    name?: string;
    icon?: React.ReactNode;
    path?: string;
    children?: AppRouter[];
    requiresPermission?: boolean;
};

interface BaseLayoutProps {
    children: React.ReactNode;
    hidden?: boolean;
}

const layoutContentStyle: CSSProperties = {
    flex: 1,
    height: "100%",
    minHeight: "0",
    overflow: "hidden",
    padding: 16,
};

const systemRoutes: AppRouter = {
    name: "System",
    icon: <SettingOutlined />,
    path: "/system",
    children: [
        {
            path: "/system/user",
            name: "User",
            icon: <UserOutlined />,
        },
        {
            path: "/system/role",
            name: "Role",
            icon: <TeamOutlined />,
        },
        {
            path: "/system/menu",
            name: "Menu",
            icon: <MenuOutlined />,
        },
        {
            path: "/system/dict",
            name: "Dictionary",
            icon: <BookOutlined />,
        },
        {
            path: "/system/log",
            name: "Log",
            icon: <HistoryOutlined />,
        },
    ],
};

const pageRoutes: AppRouter[] = [
    {
        path: "/403",
        name: "403 Page",
        icon: <StopOutlined />,
        requiresPermission: false,
    },
    {
        path: "/404",
        name: "404 Page",
        icon: <FileUnknownOutlined />,
        requiresPermission: false,
    },
    systemRoutes,
];

const getMenuData = (checkMenuPermissions: (path: string) => boolean): AppRouter[] => {
    const getMenuList = (menuList: AppRouter[]): AppRouter[] => {
        return menuList
            .filter((item) => {
                if (!item.path) return false;
                if (item.requiresPermission === false) return true;
                if (item.children) return true;
                return checkMenuPermissions(item.path);
            })
            .map((item) => ({
                ...item,
                children: item.children ? getMenuList(item.children) : undefined,
            }))
            .filter((item) => {
                if (item.children?.length === 0) {
                    return false;
                }
                return true;
            });
    };

    return getMenuList(pageRoutes);
};

export const BaseLayout = ({ children, hidden = false }: BaseLayoutProps) => {
    const userInfo = useAuthStore((state) => state.userInfo);
    const clearAuth = useAuthStore((state) => state.clearAuth);
    const checkMenuPermissions = useAuthStore((state) => state.checkMenuPermissions);
    const menuPermissionSignature = useAuthStore((state) => state.userInfo?.permissions?.join("|") || "");
    const router = useRouter();
    const currentPath = useLocation().pathname;

    const menuData = useMemo(
        () => getMenuData(checkMenuPermissions),
        [checkMenuPermissions, menuPermissionSignature],
    );

    if (hidden) {
        return children;
    }

    const userMenuItems: MenuProps["items"] = [
        {
            key: "profile",
            icon: <UserOutlined />,
            label: <Link to="/profile">Profile</Link>,
        },
        {
            type: "divider",
        },
        {
            key: "logout",
            icon: <LogoutOutlined />,
            label: "Logout",
            onClick: async () => {
                await authAPI.logout();
                clearAuth();
                appMessage.success("Logout successful");
                void router.navigate({ to: "/login" });
                return true;
            },
        },
    ];

    return (
        <ProLayout
            title="Rustzen Admin"
            logo="/rustzen.png"
            location={{ pathname: currentPath }}
            layout="mix"
            contentStyle={layoutContentStyle}
            onMenuHeaderClick={() => void router.navigate({ to: "/" })}
            menuItemRender={(item, dom) => (
                <Link to={item.path || "/"} className="block">
                    {dom}
                </Link>
            )}
            route={{
                path: "/",
                children: [
                    {
                        path: "/",
                        name: "Dashboard",
                        icon: <DashboardOutlined />,
                    },
                    ...menuData,
                ],
            }}
            avatarProps={{
                src: userInfo?.avatarUrl,
                size: "small",
                title: userInfo?.realName || userInfo?.username,
                render: (_props, dom) => {
                    return <Dropdown menu={{ items: userMenuItems }}>{dom}</Dropdown>;
                },
            }}
        >
            {children}
        </ProLayout>
    );
};
