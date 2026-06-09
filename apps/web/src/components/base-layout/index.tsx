import { LogoutOutlined, UserOutlined } from "@ant-design/icons";
import { ProLayout } from "@ant-design/pro-components";
import { Link, useLocation, useRouter } from "@tanstack/react-router";
import type { MenuProps } from "antd";
import { Dropdown } from "antd";
import { useMemo, type CSSProperties, type ReactNode } from "react";

import { appMessage, authAPI } from "@/api";
import { useAuthStore } from "@/store/useAuthStore";

import { AppSearch } from "./app-search";
import { getMenuData, getSearchRouteItems, type AppRoutePath } from "./routes";

interface BaseLayoutProps {
    children: ReactNode;
    hidden?: boolean;
}

const layoutContentStyle: CSSProperties = {
    flex: 1,
    height: "100%",
    minHeight: "0",
    overflow: "hidden",
    padding: 16,
};

export const BaseLayout = ({ children, hidden = false }: BaseLayoutProps) => {
    const userInfo = useAuthStore((state) => state.userInfo);
    const clearAuth = useAuthStore((state) => state.clearAuth);
    const checkMenuPermissions = useAuthStore((state) => state.checkMenuPermissions);
    const menuPermissionSignature = useAuthStore(
        (state) => state.userInfo?.permissions?.join("|") || "",
    );
    const router = useRouter();
    const currentPath = useLocation().pathname;

    const menuData = useMemo(
        () => getMenuData(checkMenuPermissions),
        [checkMenuPermissions, menuPermissionSignature],
    );

    const searchRoutes = useMemo(
        () => getSearchRouteItems(checkMenuPermissions),
        [checkMenuPermissions, menuPermissionSignature],
    );

    const handleSearchSelect = (path: AppRoutePath) => {
        void router.navigate({ to: path });
    };

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
                children: menuData,
            }}
            actionsRender={() => [
                <AppSearch key="page-search" routes={searchRoutes} onSelect={handleSearchSelect} />,
            ]}
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
