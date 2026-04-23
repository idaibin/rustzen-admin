import {
    BookOutlined,
    DashboardOutlined,
    HistoryOutlined,
    LogoutOutlined,
    MenuOutlined,
    SettingOutlined,
    TeamOutlined,
    UserOutlined,
} from "@ant-design/icons";
import { ProLayout } from "@ant-design/pro-components";
import { Link, useLocation, useRouter } from "@tanstack/react-router";
import type { MenuProps } from "antd";
import { Dropdown } from "antd";

import { appMessage, authAPI } from "@/api";
import { UserProfileModal } from "@/components/base-user";
import { useAuthStore } from "@/store/useAuthStore";

type AppRouter = {
    name?: string;
    icon?: React.ReactNode;
    path?: string;
    children?: AppRouter[];
};

interface BaseLayoutProps {
    children: React.ReactNode;
    hidden?: boolean;
}

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

const pageRoutes: AppRouter[] = [systemRoutes];

const getMenuData = (): AppRouter[] => {
    const { checkMenuPermissions } = useAuthStore.getState();

    const getMenuList = (menuList: AppRouter[]): AppRouter[] => {
        return menuList
            .filter((item) => {
                if (!item.path) return false;
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
    const { userInfo } = useAuthStore();
    const router = useRouter();
    const currentPath = useLocation().pathname;

    if (hidden) {
        return children;
    }

    const userMenuItems: MenuProps["items"] = [
        {
            key: "profile",
            icon: <UserOutlined />,
            label: <UserProfileModal />,
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
                useAuthStore.getState().clearAuth();
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
                    ...getMenuData(),
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
