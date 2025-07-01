import { ProLayout } from "@ant-design/pro-components";
import { Link, Outlet, useLocation, useNavigate } from "react-router-dom";
import { Dropdown, message } from "antd";
import type { MenuProps } from "antd";
import {
  SettingOutlined,
  UserOutlined,
  TeamOutlined,
  MenuOutlined,
  BookOutlined,
  HistoryOutlined,
  LogoutOutlined,
} from "@ant-design/icons";
import { useAuthStore } from "../stores/useAuthStore";
import { authAPI } from "@/services";

const BasicLayout = () => {
  const location = useLocation();
  const navigate = useNavigate();
  const { userInfo, clearAuth } = useAuthStore();
  // Handle user logout and show message
  const handleLogout = async () => {
    await authAPI.logout();
    clearAuth();
    message.success("Logout successful");
  };

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
      onClick: handleLogout,
    },
  ];

  return (
    <div style={{ height: "100vh" }}>
      {/* Main layout with sidebar and header */}
      <ProLayout
        // siderMenuType="group"
        // menu={{
        //   type: "group",
        // }}
        // fixSiderbar={true}
        // splitMenus={true}
        title="Rustzen Admin"
        logo="/rustzen.png"
        location={location}
        menuItemRender={(item, dom) => <Link to={item.path || "/"}>{dom}</Link>}
        layout="mix"
        onMenuHeaderClick={() => {
          console.log("onMenuHeaderClick");
          navigate("/");
        }}
        avatarProps={{
          src: userInfo?.avatarUrl,
          size: "small",
          title: userInfo?.realName || userInfo?.username,
          render: (props, dom) => {
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
          routes: [
            {
              path: "/system",
              name: "System Management",
              icon: <SettingOutlined />,
              routes: [
                {
                  path: "/system/user",
                  name: "User Management",
                  icon: <UserOutlined />,
                },
                {
                  path: "/system/role",
                  name: "Role Management",
                  icon: <TeamOutlined />,
                },
                {
                  path: "/system/menu",
                  name: "Menu Management",
                  icon: <MenuOutlined />,
                },
                {
                  path: "/system/dict",
                  name: "Dictionary Management",
                  icon: <BookOutlined />,
                },
                {
                  path: "/system/log",
                  name: "Log Management",
                  icon: <HistoryOutlined />,
                },
              ],
            },
          ],
        }}
      >
        {/* Main content area */}
        <main className="p-4">
          <Outlet />
        </main>
      </ProLayout>
    </div>
  );
};

export default BasicLayout;
