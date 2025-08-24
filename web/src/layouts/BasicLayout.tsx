import { ProLayout } from "@ant-design/pro-components";
import { Link, Outlet, useLocation, useNavigate } from "react-router-dom";
import { Dropdown } from "antd";
import type { MenuProps } from "antd";
import { UserOutlined, LogoutOutlined } from "@ant-design/icons";
import { useAuthStore } from "../stores/useAuthStore";
import { authAPI } from "@/api/auth";
import { getMenuData } from "@/router";
import { messageApi } from "@/main";
import { UserProfileModal } from "@/components/user";

// User dropdown menu items
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
            messageApi.success("Logout successful");
        },
    },
];

export const BasicLayout = () => {
    const location = useLocation();
    const navigate = useNavigate();
    const { userInfo } = useAuthStore();

    return (
        <ProLayout
            title="Rustzen Admin"
            logo="/rustzen.png"
            location={location}
            layout="mix"
            onMenuHeaderClick={() => navigate("/")}
            menuItemRender={(item, dom) => (
                <Link to={item.path || "/"}>{dom}</Link>
            )}
            route={{
                path: "/",
                children: getMenuData(),
            }}
            avatarProps={{
                src: userInfo?.avatarUrl,
                size: "small",
                title: userInfo?.realName || userInfo?.username,
                render: (_props, dom) => {
                    return (
                        <Dropdown menu={{ items: userMenuItems }}>
                            {dom}
                        </Dropdown>
                    );
                },
            }}
        >
            <Outlet />
        </ProLayout>
    );
};
