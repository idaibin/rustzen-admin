import { ProLayout } from "@ant-design/pro-components";
import { Link, Outlet, useLocation, useNavigate } from "react-router-dom";
import { Dropdown, message } from "antd";
import type { MenuProps } from "antd";
import { UserOutlined, LogoutOutlined } from "@ant-design/icons";
import { useAuthStore } from "../stores/useAuthStore";
import { authAPI } from "@/services";
import { getMenuData } from "@/router";

// User dropdown menu items
const userMenuItems: MenuProps["items"] = [
    {
        key: "profile",
        icon: <UserOutlined />,
        label: "Profile",
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
            message.success("Logout successful");
        },
    },
];

const BasicLayout = () => {
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
            avatarProps={{
                src: userInfo?.avatarUrl,
                size: "small",
                title: userInfo?.realName || userInfo?.username,
                render: (_props, dom) => {
                    return (
                        <Dropdown
                            menu={{
                                items: userMenuItems,
                            }}
                        >
                            {dom}
                        </Dropdown>
                    );
                },
            }}
            route={{
                path: "/",
                children: getMenuData(),
            }}
            // route={{
            //     path: "/",
            //     children: [
            //         {
            //             path: "/system",
            //             name: "System",
            //             icon: <SettingOutlined />,
            //             children: [
            //                 {
            //                     path: "/system/user",
            //                     name: "User",
            //                     icon: <UserOutlined />,
            //                 },
            //                 {
            //                     path: "/system/role",
            //                     name: "Role",
            //                     icon: <TeamOutlined />,
            //                 },
            //                 {
            //                     path: "/system/menu",
            //                     name: "Menu",
            //                     icon: <MenuOutlined />,
            //                 },
            //                 {
            //                     path: "/system/dict",
            //                     name: "Dictionary",
            //                     icon: <BookOutlined />,
            //                 },
            //                 {
            //                     path: "/system/log",
            //                     name: "Log",
            //                     icon: <HistoryOutlined />,
            //                 },
            //             ],
            //         },
            //     ],
            // }}
        >
            <main className="p-4">
                <Outlet />
            </main>
        </ProLayout>
    );
};

export default BasicLayout;
