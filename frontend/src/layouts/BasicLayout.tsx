import { ProLayout } from "@ant-design/pro-components";
import { Link, Outlet, useLocation } from "react-router-dom";
import {
  SettingOutlined,
  UserOutlined,
  TeamOutlined,
  MenuOutlined,
  BookOutlined,
  HistoryOutlined,
} from "@ant-design/icons";

const BasicLayout = () => {
  const location = useLocation();

  return (
    <div style={{ height: "100vh" }}>
      <ProLayout
        title="Rust-Zen-Admin"
        logo="/vite.svg"
        location={location}
        route={{
          path: "/",
          routes: [
            {
              path: "/system",
              name: "系统管理",
              icon: <SettingOutlined />,
              routes: [
                {
                  path: "/system/user",
                  name: "用户管理",
                  icon: <UserOutlined />,
                },
                {
                  path: "/system/role",
                  name: "角色管理",
                  icon: <TeamOutlined />,
                },
                {
                  path: "/system/menu",
                  name: "菜单管理",
                  icon: <MenuOutlined />,
                },
                {
                  path: "/system/dict",
                  name: "字典管理",
                  icon: <BookOutlined />,
                },
                {
                  path: "/system/log",
                  name: "日志管理",
                  icon: <HistoryOutlined />,
                },
              ],
            },
          ],
        }}
        menuItemRender={(item, dom) => <Link to={item.path || "/"}>{dom}</Link>}
      >
        <main className="p-4">
          <Outlet />
        </main>
      </ProLayout>
    </div>
  );
};

export default BasicLayout;
